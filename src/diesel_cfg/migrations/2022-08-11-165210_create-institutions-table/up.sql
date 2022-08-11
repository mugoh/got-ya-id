CREATE TABLE IF NOT EXISTS institutions (
    id serial PRIMARY KEY,
    name TEXT NOT NULL,
    town TEXT NOT NULL,
    country TEXT NOT NULL,
    description TEXT,
    postal_address TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
