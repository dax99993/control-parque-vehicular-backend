-- Add up migration script here

CREATE TYPE usuario_rol AS ENUM ('admin', 'normal');

CREATE TABLE IF NOT EXISTS usuarios
(
    usuario_id uuid NOT NULL PRIMARY KEY,
    nombres TEXT NOT NULL,
    apellidos TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    imagen TEXT NOT NULL DEFAULT 'users/default-picture.jpeg',
    numero_empleado SMALLINT NULL,
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    verificado BOOLEAN NOT NULL DEFAULT FALSE,
    departamento INTEGER NULL DEFAULT 1 REFERENCES departamentos(id) ON DELETE SET NULL,
    --departamento INTEGER NULL REFERENCES departamentos(id) ON DELETE SET NULL,
    rol usuario_rol DEFAULT 'normal',
    creado_en TIMESTAMP NOT NULL DEFAULT NOW(),
    modificado_en TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX usuarios_email_idx ON usuarios (email);

