
CREATE TABLE oath_users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(40) NOT NULL,
    name VARCHAR NOT NULL,
    first_name VARCHAR(50),
    family_name VARCHAR(50),
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    picture VARCHAR,
    locale VARCHAR(5),
    acc_id VARCHAR NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    provider VARCHAR(15) NOT NULL,

    created_at timestamp without time zone not null default (now() at time zone 'utc'), 
    updated_at timestamp without time zone not null default (now() at time zone 'utc')
);
