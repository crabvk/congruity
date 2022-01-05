set dotenv-load := true

fetch-deps:
    git submodule update --init --recursive --checkout

run:
    env RUST_LOG=congruity=debug cargo run

deploy:
    cargo build
    rsync -az target/debug/congruity $SERVER:~/congruity

db:
    @psql $POSTGRESQL_URL

redis:
    @redis-cli -u $REDIS_URL

client *args:
    #!/usr/bin/env fish
    set addr (string trim -lc 'http://' $CONCORDIUM_GRPC_URL | string split ':')
    concordium-client \
        --grpc-ip $addr[1] \
        --grpc-port $addr[2] \
        --grpc-authentication-token $CONCORDIUM_GRPC_TOKEN \
        {{args}}

search:
    #!/usr/bin/env fish
    set dir (fd -td -d1 '^github\.com-\w{16}' ~/.cargo/registry/src)
    cd $dir
    set pkg (sk -c 'fd -td -d1')
    test $status -eq 0 && echo $dir/(basename $pkg)
