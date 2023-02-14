-- Add up migration script here
CREATE TABLE IF NOT EXISTS signup_tokens (
    signup_token TEXT NOT NULL,
    user_id uuid NOT NULL
        REFERENCES users(user_id),
    PRIMARY KEY(signup_token)
);

