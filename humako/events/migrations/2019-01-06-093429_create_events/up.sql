CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE events (
  id uuid DEFAULT uuid_generate_v4(),
  unique_id SERIAL UNIQUE,
  event_type VARCHAR NOT NULL,
  payload TEXT NOT NULL,
  timestamp TIMESTAMP DEFAULT Now() NOT NULL,
  PRIMARY KEY (id)
);