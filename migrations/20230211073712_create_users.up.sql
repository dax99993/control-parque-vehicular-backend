-- Add up migration script here

CREATE TABLE IF NOT EXISTS users 
(
    user_id uuid NOT NULL PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    picture TEXT NOT NULL DEFAULT 'users/default-picture.jpeg',
    employee_number SMALLINT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    department INTEGER NULL REFERENCES departments(id) ON DELETE SET NULL,
    role TEXT NOT NULL DEFAULT 'normal' CHECK( role IN ('normal', 'admin') ),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX users_email_idx ON users (email);

