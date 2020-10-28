CREATE TABLE IF NOT EXISTS tokens
  ( id UUID DEFAULT uuid_generate_v4() NOT NULL
  , user_id UUID NOT NULL
  , created_at TIMESTAMP NOT NULL DEFAULT NOW()
  , updated_at TIMESTAMP NOT NULL DEFAULT NOW()
  , deleted_at TIMESTAMP
  , PRIMARY KEY (id)
  );

CREATE TRIGGER set_timestamp_tokens
  BEFORE UPDATE ON tokens
  FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
