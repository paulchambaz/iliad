CREATE TABLE IF NOT EXISTS users (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audiobooks (
    hash TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    date INTEGER NOT NULL,
    description TEXT NOT NULL,
    genres TEXT NOT NULL,
    duration INTEGER NOT NULL,
    size INTEGER NOT NULL,
    path TEXT NOT NULL UNIQUE,
    final_chapter_index INTEGER,
    final_chapter_position INTEGER,
    archive_checksum TEXT,
    archive_ready INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS positions (
    audiobook_hash TEXT NOT NULL,
    username TEXT NOT NULL,
    chapter_index INTEGER NOT NULL,
    chapter_position INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (audiobook_hash, username)
);
