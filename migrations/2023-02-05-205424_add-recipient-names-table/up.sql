-- Your SQL goes here
CREATE TABLE recipients (
    fullname TEXT PRIMARY KEY NOT NULL,
    is_real BOOLEAN NOT NULL DEFAULT FALSE
)