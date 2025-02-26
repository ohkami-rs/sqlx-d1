-- Migration number: 0001 	 2025-02-24T17:10:24.835Z

CREATE TABLE users (
    id   INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    age  INTEGER
);

CREATE TABLE todos (
    id          INTEGER NOT NULL PRIMARY KEY,
    user_id     INTEGER NOT NULL,
    title       TEXT NOT NULL,
    description TEXT
);
