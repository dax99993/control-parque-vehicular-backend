-- Add up migration script here

CREATE TYPE estado_vehiculo AS ENUM ('disponible', 'ocupado', 'mantenimiento');

CREATE TABLE IF NOT EXISTS vehiculos
(
    vehiculo_id uuid NOT NULL PRIMARY KEY,
    marca TEXT NOT NULL,
    modelo TEXT NOT NULL,
    año SMALLINT NOT NULL,
    CHECK (año > 0),
    numero_placa TEXT NOT NULL DEFAULT '',
    nombre_economico TEXT NOT NULL DEFAULT '',
    numero_tarjeta TEXT NOT NULL DEFAULT '',
    --latitud DECIMAL(9, 6) DEFAULT 22.761202,
    --longitud DECIMAL(9, 6) DEFAULT -102.579088,
    --estado TEXT NOT NULL DEFAULT 'available' CHECK( estado IN ('available', 'occupied', 'maintenance') ),
    --kilometraje INT NOT NULL DEFAULT 0,
    estado estado_vehiculo DEFAULT 'disponible',
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    imagen TEXT NOT NULL DEFAULT 'default-vehicule.jpeg',
    creado_en TIMESTAMP NOT NULL DEFAULT NOW(),
    modificado_en TIMESTAMP NOT NULL DEFAULT NOW()
    --CONSTRAINT bounded_latitude CHECK (latitude >= -90.0 AND latitude < 90.0),
    --CONSTRAINT bounded_longitude CHECK (latitude >= -180.0 AND latitude < 180.0)
);


INSERT INTO vehiculos (vehiculo_id, marca, modelo, año, numero_placa, nombre_economico, numero_tarjeta)
VALUES
('5dfa50d4-9ecf-4a53-80b4-a3468c0ef9d1', 'Nissan', 'Zakura', 2015, 'ABCD XYZ', 'Zaku 12', 'tarjeta01'),
('fefa3ab9-2ad0-4c01-9959-c18bce2f5aed', 'Nissan', 'Tsuru', 1999, '1234 XYZ', 'Tsurito 4', 'tarjeta02'),
('32587faf-a145-42c7-8f60-edb2727a1834', 'Nissan', 'Sentra', 1998, 'XYZ QWER', 'Sentra 23', 'tarjeta03'),
('1dc8e9a0-e2e1-4a1d-94f3-7b51276376be', 'Toyota', 'Avalon', 2018, 'ABC 123', 'Ava 01', 'tarjeta04'),
('f5bf8492-24bb-4739-8052-1d4e91c7be94', 'Toyota', 'Etios', 2011, 'Placa 123', 'Etios 04', 'tarjeta05');
