fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_server(false).compile(
        &["deps/grpc-api/concordium_p2p_rpc.proto"],
        &["deps/grpc-api"],
    )?;
    println!("cargo:rerun-if-changed=migrations");
    Ok(())
}
