CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL ,
    phone VARCHAR,
    name VARCHAR,
    institution VARCHAR,
    about TEXT,
    found_ids INTEGER
);

CREATE UNIQUE INDEX user_profile_id on profiles (user_id);
