-- Your SQL goes here
CREATE TABLE AVATARS (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    url VARCHAR(1024)
)
