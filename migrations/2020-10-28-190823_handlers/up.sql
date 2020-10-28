CREATE TABLE IF NOT EXISTS handlers
  ( id UUID DEFAULT uuid_generate_v4() NOT NULL
  , user_id UUID NOT NULL
  , human_name VARCHAR NOT NULL
  , current_version VARCHAR NOT NULL
  , async_impl BOOLEAN DEFAULT false
  , created_at TIMESTAMP NOT NULL DEFAULT NOW()
  , updated_at TIMESTAMP NOT NULL DEFAULT NOW()
  , PRIMARY KEY (id)
  , CONSTRAINT fk_user_id
    FOREIGN KEY (user_id)
    REFERENCES users(id)
  );

CREATE TRIGGER set_timestamp_handlers
  BEFORE UPDATE ON handlers
  FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
