CREATE TABLE USERS (
    id SERIAL PRIMARY KEY,
    username VARCHAR  NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    phone VARCHAR,
    firstname VARCHAR,
    middlename VARCHAR,
    lastname VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT LOCALTIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE

)
