-- Add up migration script here
CREATE TABLE IF NOT EXISTS peticiones
(
    peticion_id UUID NOT NULL PRIMARY KEY,
    usuario_id uuid NULL REFERENCES usuarios(usuario_id) ON DELETE SET NULL,
    vehiculo_id uuid NULL REFERENCES vehiculos(vehiculo_id) ON DELETE SET NULL,
    estado TEXT NOT NULL DEFAULT 'pending' CHECK( estado IN ('approved', 'denied', 'pending', 'finished') ),
    actividad_descripcion TEXT NOT NULL DEFAULT '',
    actividad_comentario TEXT NOT NULL DEFAULT '',
    --feedback_comment TEXT NOT NULL DEFAULT '',
    kilometraje_inicial SMALLINT NOT NULL,
    CHECK (kilometraje_inicial > 0),
    kilometraje_final SMALLINT NOT NULL,
    CHECK (kilometraje_final > 0),
    CONSTRAINT valid_milage_final CHECK (kilometraje_final >= kilometraje_inicial),
    usuario_licencia_imagen TEXT NOT NULL,
    vehiculo_imagen TEXT NOT NULL,
    gasolina_imagen TEXT NULL,
    creado_en TIMESTAMP NOT NULL DEFAULT NOW(),
    modificado_en TIMESTAMP NOT NULL DEFAULT NOW()
);
