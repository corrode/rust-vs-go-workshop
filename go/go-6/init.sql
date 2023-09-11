CREATE TABLE IF NOT EXISTS cities (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    lat NUMERIC NOT NULL,
    long NUMERIC NOT NULL
);

CREATE INDEX IF NOT EXISTS cities_name_idx ON cities (name);
