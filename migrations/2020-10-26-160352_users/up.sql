CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE OR REPLACE FUNCTION trigger_set_timestamp()
  RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS users
  ( id UUID DEFAULT uuid_generate_v4() NOT NULL
  , email VARCHAR UNIQUE NOT NULL
  , salutation VARCHAR NOT NULL
  , is_admin BOOLEAN DEFAULT false NOT NULL
  , is_locked BOOLEAN DEFAULT false NOT NULL
  , tier INTEGER DEFAULT 0 NOT NULL
  , created_at TIMESTAMP NOT NULL DEFAULT NOW()
  , updated_at TIMESTAMP NOT NULL DEFAULT NOW()
  , PRIMARY KEY (id)
  );

CREATE TRIGGER set_timestamp_users
  BEFORE UPDATE ON users
  FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
