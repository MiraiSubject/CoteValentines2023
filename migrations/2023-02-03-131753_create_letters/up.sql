CREATE TABLE letters (
  id INTEGER PRIMARY KEY,

  recipient VARCHAR NOT NULL,
  sender VARCHAR NOT NULL,
  anon BOOLEAN NOT NULL DEFAULT FALSE,

  content TEXT NOT NULL
)
