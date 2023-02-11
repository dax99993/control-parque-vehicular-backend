-- Add up migration script here
CREATE TYPE user_role AS ENUM ('normal', 'admin');

CREATE TABLE IF NOT EXISTS users 
(
    id uuid NOT NULL PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    picture TEXT NOT NULL,
    employee_number SMALLINT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    department INTEGER NULL REFERENCES departments(id) ON DELETE SET NULL,
    role user_role NOT NULL DEFAULT 'normal',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX users_email_idx ON users (email);

