-- Your SQL goes here
CREATE TABLE USERS (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    phone VARCHAR,
    first_name VARCHAR,
    middle_name VARCHAR,
    last_name VARCHAR,
    created_at timestamp without time zone not null default (now() at time zone 'utc'), 
    updated_at timestamp without time zone not null default (now() at time zone 'utc'),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE

);

CREATE UNIQUE INDEX user_creds on USERS (username, email);
