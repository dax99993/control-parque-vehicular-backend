-- Add up migration script here
CREATE TABLE IF NOT EXISTS departamentos
(
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL
);

INSERT INTO departamentos
(nombre) VALUES
('Laboratorio de Software Libre'),
('Recursos Humanos'),
('Becas');
