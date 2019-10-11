-- Your SQL goes here
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    institution VARCHAR NOT NULL DEFAULT 'unknown',
    phone VARCHAR,
    avatar VARCHAR,
    found_ids INTEGER
    )
