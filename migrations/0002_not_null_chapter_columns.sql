CREATE TABLE audiobooks_new (
    hash TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    date INTEGER NOT NULL,
    description TEXT NOT NULL,
    genres TEXT NOT NULL,
    duration INTEGER NOT NULL,
    size INTEGER NOT NULL,
    path TEXT NOT NULL UNIQUE,
    final_chapter_index INTEGER NOT NULL DEFAULT 0,
    final_chapter_position INTEGER NOT NULL DEFAULT 0,
    archive_checksum TEXT,
    archive_ready INTEGER NOT NULL DEFAULT 0
);

INSERT INTO audiobooks_new SELECT * FROM audiobooks;

DROP TABLE audiobooks;

ALTER TABLE audiobooks_new RENAME TO audiobooks;
