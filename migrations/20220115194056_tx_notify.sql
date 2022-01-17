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
                SELECT json_build_object(
                    'index_id', NEW.id,
                    'account', NEW.account,
                    'summary', summary
                ) AS payload FROM summary
            )
            SELECT pg_notify(TG_ARGV[0], payload::text) FROM payload
        );
    END IF;

    RETURN NULL;
END;
$$;
