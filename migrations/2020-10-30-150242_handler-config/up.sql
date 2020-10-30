CREATE TABLE IF NOT EXISTS handler_config
  ( key_name VARCHAR NOT NULL
  , value_contents VARCHAR NOT NULL
  , handler_id UUID NOT NULL
  , created_at TIMESTAMP NOT NULL DEFAULT NOW()
  , updated_at TIMESTAMP NOT NULL DEFAULT NOW()
  , PRIMARY KEY (key_name, handler_id)
  , CONSTRAINT fk_handler_id
    FOREIGN KEY (handler_id)
    REFERENCES handlers(id)
  );

CREATE TRIGGER set_timestamp_handler_config
  BEFORE UPDATE ON handler_config
  FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
