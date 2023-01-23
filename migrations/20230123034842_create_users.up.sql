-- Add up migration script here

CREATE TABLE IF NOT EXISTS departments (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TYPE user_status AS ENUM ('normal', 'admin');

CREATE TABLE IF NOT EXISTS users 
(
    id SERIAL PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    picture TEXT NOT NULL DEFAULT 'static/user.jpg',
    employee_number SMALLINT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    department INTEGER NULL REFERENCES departments(id) ON DELETE SET NULL,
    status user_status NOT NULL DEFAULT 'normal',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO departments
(name) VALUES
('Laboratorio de Software Libre'),
('Recursos Humanos'),
('Becas');

INSERT INTO users 
(first_name, last_name, email, password, status) VALUES
('Daniel', 'Ban Torres', 'dax99993@gmail.com', 'pass1234', 'admin');


