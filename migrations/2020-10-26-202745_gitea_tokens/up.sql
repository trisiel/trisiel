CREATE TABLE IF NOT EXISTS gitea_tokens
  ( id UUID DEFAULT uuid_generate_v4() NOT NULL
  , user_id UUID NOT NULL
  , access_token VARCHAR NOT NULL
  , refresh_token VARCHAR NOT NULL
  , created_at TIMESTAMP NOT NULL DEFAULT NOW()
  , updated_at TIMESTAMP NOT NULL DEFAULT NOW()
  , PRIMARY KEY (id)
  , CONSTRAINT fk_user_id
    FOREIGN KEY (user_id)
    REFERENCES users(id)
  );

CREATE TRIGGER set_timestamp_gitea_tokens
  BEFORE UPDATE ON gitea_tokens
  FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
