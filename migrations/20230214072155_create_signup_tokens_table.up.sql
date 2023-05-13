-- Add up migration script here
CREATE TABLE IF NOT EXISTS signup_tokens (
    signup_token TEXT NOT NULL,
    usuario_id uuid NOT NULL
        REFERENCES usuarios(usuario_id) ON DELETE CASCADE,
    PRIMARY KEY(signup_token)
);

