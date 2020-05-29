CREATE TABLE emails (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    removed BOOLEAN NOT NULL DEFAULT FALSE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    
    created_at timestamp without time zone not null default (now() at time zone 'utc'),
    updated_at timestamp without time zone not null default (now() at time zone 'utc')
);

ALTER TABLE users DROP COLUMN IF EXISTS email;

ALTER TABLE users DROP CONSTRAINT IF EXISTS user_creds;

SELECT diesel_manage_updated_at('emails');
