
CREATE TYPE estado_peticion AS ENUM ('pendiente', 'aceptada', 'rechazada', 'finalizada');

-- Add up migration script here
CREATE TABLE IF NOT EXISTS peticiones
(
    peticion_id UUID NOT NULL PRIMARY KEY,
    usuario_id uuid NOT NULL REFERENCES usuarios(usuario_id) ON DELETE CASCADE,
    vehiculo_id uuid NOT NULL REFERENCES vehiculos(vehiculo_id) ON DELETE CASCADE,
    estado estado_peticion DEFAULT 'pendiente',
    inicio TIMESTAMP NOT NULL,
    finalizo TIMESTAMP NOT NULL,
    actividad_descripcion TEXT NOT NULL DEFAULT '',
    actividad_comentario TEXT NOT NULL DEFAULT '',
    kilometraje_inicial INT NOT NULL,
    CHECK (kilometraje_inicial > 0),
    kilometraje_final INT NOT NULL,
    CHECK (kilometraje_final > 0),
    CONSTRAINT kilometraje_final_valido CHECK (kilometraje_final >= kilometraje_inicial),
    usuario_licencia_imagen TEXT NOT NULL DEFAULT '',
    vehiculo_imagen TEXT NOT NULL DEFAULT '',
    gasolina_imagen TEXT NOT NULL DEFAULT '',
    creado_en TIMESTAMP NOT NULL DEFAULT NOW(),
    modificado_en TIMESTAMP NOT NULL DEFAULT NOW()
);


INSERT INTO peticiones (peticion_id, vehiculo_id, usuario_id, inicio, finalizo, kilometraje_inicial, kilometraje_final, usuario_licencia_imagen)
VALUES 
('6dafcf4c-4582-4319-b2e7-11971104abf9',
    'fefa3ab9-2ad0-4c01-9959-c18bce2f5aed',
    'b54d1954-7c59-4907-b9c6-41db193c716c',
    '2023-06-20T06:20:00Z',
    '2023-06-20T07:30:00Z',
    200000,
    300000,
    ''
),
('50f344d0-a3c2-45bb-bdac-7c76f26daf9f',
    '1dc8e9a0-e2e1-4a1d-94f3-7b51276376be',
    'b54d1954-7c59-4907-b9c6-41db193c716c',
    '2023-06-22T09:30:00Z',
    '2023-06-22T12:30:00Z',
    200100,
    200150,
    '');
