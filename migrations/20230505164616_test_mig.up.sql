-- Add up migration script here

CREATE TABLE IF NOT EXISTS test
(
    frase TEXT NOT NULL,
    año SMALLINT NOT NULL
);
