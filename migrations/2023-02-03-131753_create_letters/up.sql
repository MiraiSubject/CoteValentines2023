CREATE TABLE letters (
  id INTEGER PRIMARY KEY NOT NULL,

  recipient VARCHAR NOT NULL,
  sender VARCHAR NOT NULL,
  anon BOOLEAN NOT NULL DEFAULT FALSE,

  content TEXT NOT NULL
)