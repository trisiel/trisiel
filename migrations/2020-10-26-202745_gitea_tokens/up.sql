CREATE TABLE IF NOT EXISTS gitea_tokens
  ( id UUID DEFAULT uuid_generate_v4() NOT NULL
  , user_id UUID NOT NULL
  , access_token VARCHAR NOT NULL
  , refresh_token VARCHAR NOT NULL
  , PRIMARY KEY (id)
  , CONSTRAINT fk_user_id
    FOREIGN KEY (user_id)
    REFERENCES users(id)
  );
