-- Add up migration script here

CREATE TYPE usuario_rol AS ENUM ('admin', 'normal');

CREATE TABLE IF NOT EXISTS usuarios
(
    usuario_id uuid NOT NULL PRIMARY KEY,
    nombres TEXT NOT NULL,
    apellidos TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    imagen TEXT NOT NULL DEFAULT 'default-user.jpeg',
    numero_empleado SMALLINT NULL,
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    verificado BOOLEAN NOT NULL DEFAULT FALSE,
    departamento INTEGER NULL DEFAULT NULL REFERENCES departamentos(id) ON DELETE SET NULL,
    --departamento INTEGER NULL REFERENCES departamentos(id) ON DELETE SET NULL,
    rol usuario_rol DEFAULT 'normal',
    creado_en TIMESTAMP NOT NULL DEFAULT NOW(),
    modificado_en TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX usuarios_email_idx ON usuarios (email);
CREATE INDEX usuarios_nombres_idx ON usuarios (nombres);
CREATE INDEX usuarios_apellidos_idx ON usuarios (apellidos);


INSERT INTO usuarios (usuario_id, nombres, apellidos, email, password_hash)
VALUES
('e9c3327a-c2ed-4945-92e0-2030225ed19a',
    'Andres Manuel',
    'Lopez Obrador',
    'cabeza_de_algodon@email.com',
    '$argon2id$v=19$m=15000,t=2,p=1$8xydI/S+x9xSFoZ8h0r9xg$m36/dVJSwRbbk6GYkkDDiaefwrmie87eAH14UMbxc+o'
),
('b54d1954-7c59-4907-b9c6-41db193c716c',
    'Manuel',
    'Sanchez Perez',
    'manuelperez@hotmail.com',
    '$argon2id$v=19$m=15000,t=2,p=1$ljYtcH3zUC9xML8SRcvq2Q$SOoKT4S2Hbd0XuaBsbjRRJSgpuzV4AHmUBoqOl94Zz0'
);
--('', 'Andres Manuel', 'Lopez Obrador', 'cabeza_de_algodon@email.com', ''),
