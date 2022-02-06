CREATE INDEX ON subscriptions (account);

CREATE OR REPLACE FUNCTION tx_notify()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF EXISTS (SELECT true FROM subscriptions WHERE account = NEW.account LIMIT 1) THEN
        PERFORM (
            WITH summary AS (
                SELECT summaries.summary FROM summaries WHERE summaries.id = NEW.summary
            ), payload AS (
                SELECT concat_ws('|', NEW.id, NEW.account, summary) AS payload FROM summary
            )
            SELECT pg_notify(TG_ARGV[0], payload) FROM payload
        );
    END IF;

    RETURN NULL;
END;
$$;
