-- Add up migration script here
CREATE TYPE request_state AS ENUM ('approved', 'denied', 'pending', 'finished');

CREATE TABLE IF NOT EXISTS requests 
(
    request_id UUID NOT NULL PRIMARY KEY,
    user_id uuid NULL REFERENCES users(user_id) ON DELETE SET NULL,
    vehicule_id uuid NULL REFERENCES vehicules(vehicule_id) ON DELETE SET NULL,
    state request_state NOT NULL DEFAULT 'pending',
    activity_desc TEXT NOT NULL DEFAULT '',
    activity_comment TEXT NOT NULL DEFAULT '',
    feedback_comment TEXT NOT NULL DEFAULT '',
    milage_initial SMALLINT NOT NULL,
    CHECK (milage_initial > 0),
    milage_final SMALLINT NOT NULL,
    CHECK (milage_final > 0),
    CONSTRAINT valid_milage_final CHECK (milage_final >= milage_initial),
    user_license_picture TEXT NOT NULL,
    vehicule_picture TEXT NOT NULL,
    gas_picture TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
