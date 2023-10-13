CREATE TABLE IF NOT EXISTS "audiobooks" (
  "hash"        TEXT NOT NULL,
  "title"       TEXT NOT NULL,
  "author"      TEXT NOT NULL,
  "date"        TEXT NOT NULL,
  "description" TEXT NOT NULL,
  "genres"      TEXT NOT NULL,
  "duration"    INT  NOT NULL,
  "size"        INT  NOT NULL,
  "path"        TEXT NOT NULL,
  PRIMARY KEY ("hash")
)
