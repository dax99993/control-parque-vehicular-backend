-- Add up migration script here
CREATE TYPE request_state AS ENUM ('approved', 'denied', 'pending', 'finished');

CREATE TABLE IF NOT EXISTS requests 
(
    id UUID PRIMARY KEY,
    user_id INTEGER NULL REFERENCES users(id) ON DELETE SET NULL,
    vehicule_id INTEGER NULL REFERENCES vehicules(id) ON DELETE SET NULL,
    state request_state NOT NULL DEFAULT 'pending',
    activity_desc TEXT NOT NULL DEFAULT '',
    activity_comment TEXT NOT NULL DEFAULT '',
    feedback_comment TEXT NOT NULL DEFAULT '',
    milage_initial SMALLINT NOT NULL,
    CHECK (milage_initial > 0),
    milage_final SMALLINT NOT NULL,
    CHECK (milage_final > 0),
    CONSTRAINT valid_milage_final CHECK (milage_final > milage_initial),
    user_license_picture TEXT NOT NULL,
    vehicule_picture TEXT NOT NULL,
    gas_picture TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

