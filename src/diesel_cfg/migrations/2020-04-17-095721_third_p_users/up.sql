
CREATE TABLE oath_users (
    id SERIAL PRIMARY KEY,
    email VARCHAR NOT NULL UNIQUE,
    name VARCHAR NOT NULL,
    first_name VARCHAR,
    family_name VARCHAR,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    picture VARCHAR,
    locale VARCHAR,
    acc_id VARCHAR NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at timestamp without time zone not null default (now() at time zone 'utc'), 
    updated_at timestamp without time zone not null default (now() at time zone 'utc')
);
