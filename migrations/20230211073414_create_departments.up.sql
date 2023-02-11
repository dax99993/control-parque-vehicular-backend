-- Add up migration script here
CREATE TABLE IF NOT EXISTS departments
(
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

INSERT INTO departments
(name) VALUES
('Laboratorio de Software Libre'),
('Recursos Humanos'),
('Becas');
