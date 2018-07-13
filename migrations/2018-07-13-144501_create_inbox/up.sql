CREATE TABLE inbox (
       rowid INTEGER NOT NULL,
       id TEXT UNIQUE NOT NULL,
       json TEXT NOT NULL,
       PRIMARY KEY (rowid)
);

CREATE INDEX inbox_id_ix ON inbox (id);
