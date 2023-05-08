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
    estado estado_vehiculo DEFAULT 'disponible',
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    imagen TEXT NOT NULL DEFAULT 'vehicules/default-picture.jpeg',
    creado_en TIMESTAMP NOT NULL DEFAULT NOW(),
    modificado_en TIMESTAMP NOT NULL DEFAULT NOW()
    --CONSTRAINT bounded_latitude CHECK (latitude >= -90.0 AND latitude < 90.0),
    --CONSTRAINT bounded_longitude CHECK (latitude >= -180.0 AND latitude < 180.0)
);
