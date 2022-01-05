CREATE TABLE IF NOT EXISTS subscriptions (
    id serial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    account bytea NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp,
    UNIQUE (user_id, account)
);

CREATE OR REPLACE FUNCTION tx_notify()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF EXISTS (SELECT true FROM subscriptions WHERE account = NEW.account LIMIT 1) THEN
        PERFORM (
            WITH summary AS (
                SELECT summaries.summary FROM summaries WHERE summaries.id = NEW.summary
            )
            SELECT pg_notify(TG_ARGV[0], summary::text) FROM summary
        );
    END IF;

    RETURN NULL;
END;
$$;

CREATE TRIGGER ati_notify_on_insert
AFTER INSERT
ON ati
FOR EACH ROW
EXECUTE PROCEDURE tx_notify('tx_channel');
