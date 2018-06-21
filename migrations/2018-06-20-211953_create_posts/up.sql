CREATE TABLE posts (
       id INTEGER NOT NULL,
       uri_name TEXT NOT NULL UNIQUE,
       datetime INTEGER NOT NULL DEFAULT (strftime('%s','now')),
       title TEXT NOT NULL,
       body BLOB NOT NULL,
       PRIMARY KEY (id)
);

CREATE INDEX posts_uri_name_ix ON posts (uri_name);
