CREATE TABLE "positions" (
  "hash"          TEXT NOT NULL,
  "username"      TEXT NOT NULL,
  "file"          TEXT NOT NULL,
  "position"      INTEGER NOT NULL,
  "last_modified" DATE NOT NULL,
  PRIMARY KEY("hash", "username")
)
