-- Your SQL goes here
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL ,
    phone VARCHAR(15),
    first_name VARCHAR(20),
    middle_name VARCHAR(20),
    last_name VARCHAR(20),
    institution VARCHAR(100),
    about TEXT,
    found_ids INTEGER
);

CREATE UNIQUE INDEX user_profile_id on profiles (user_id);
