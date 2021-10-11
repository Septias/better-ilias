-- Your SQL goes here
CREATE TABLE notes (
  uri TEXT NOT NULL PRIMARY KEY,
  course VARCHAR NOT NULL,
  body TEXT NOT NULL Default ""
)