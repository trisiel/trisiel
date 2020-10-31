CREATE TABLE IF NOT EXISTS executions
  ( id UUID DEFAULT uuid_generate_v4() NOT NULL
  , handler_id UUID NOT NULL
  , finished BOOLEAN NOT NULL DEFAULT false
  , stderr VARCHAR
  , created_at TIMESTAMP NOT NULL DEFAULT NOW()
  , updated_at TIMESTAMP NOT NULL DEFAULT NOW()
  , execution_time INTEGER -- XXX(Cadey): change this when you need handlers to run longer than 22 days
  , PRIMARY KEY (id)
  , CONSTRAINT fk_handler_id
    FOREIGN KEY (handler_id)
    REFERENCES handlers(id)
  );

CREATE TRIGGER set_timestamp_executions
  BEFORE UPDATE ON executions
  FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
