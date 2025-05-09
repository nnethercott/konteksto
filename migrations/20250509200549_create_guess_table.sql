-- Add migration script here
CREATE TABLE IF NOT EXISTS guesses(
  word TEXT PRIMARY KEY NOT NULL, 
  score int
);
