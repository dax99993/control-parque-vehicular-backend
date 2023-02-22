-- Add up migration script here

CREATE TABLE IF NOT EXISTS vehicules
(
    vehicule_id uuid NOT NULL PRIMARY KEY,
    branch TEXT NOT NULL,
    model TEXT NOT NULL,
    year SMALLINT NOT NULL,
    CHECK (year > 0),
    number_plate TEXT NOT NULL DEFAULT '',
    short_name TEXT NOT NULL DEFAULT '',
    number_card TEXT NOT NULL DEFAULT '',
    --latitude DECIMAL(9, 6) DEFAULT 22.761202,
    --longitude DECIMAL(9, 6) DEFAULT -102.579088,
    status TEXT NOT NULL DEFAULT 'available' CHECK( status IN ('available', 'occupied', 'maintenance') ),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    picture TEXT NOT NULL DEFAULT 'uploads/vehicules/default-picture.jpeg',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
    --CONSTRAINT bounded_latitude CHECK (latitude >= -90.0 AND latitude < 90.0),
    --CONSTRAINT bounded_longitude CHECK (latitude >= -180.0 AND latitude < 180.0)
);
