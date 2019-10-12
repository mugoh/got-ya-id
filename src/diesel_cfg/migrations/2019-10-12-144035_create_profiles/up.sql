-- Your SQL goes here
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL ,
    institution VARCHAR,
    phone VARCHAR,
    avatar VARCHAR,
    found_ids INTEGER
);

CREATE UNIQUE INDEX user_profile_id on profiles (user_id);
