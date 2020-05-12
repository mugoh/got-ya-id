CREATE TABLE claimed_identifications (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES users (id) ON DELETE CASCADE NOT NULL,
    name VARCHAR NOT NULL,
    course VARCHAR NOT NULL,
    entry_year DATE,
    graduation_year DATE,
    institution VARCHAR NOT NULL,
    campus_location VARCHAR NOT NULL,
    created_at timestamp without time zone not null default (now() at time zone 'utc'),
    updated_at timestamp without time zone not null default (now() at time zone 'utc')
);

SELECT diesel_manage_updated_at('claimed_identifications');
