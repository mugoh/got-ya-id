ALTER TABLE identifications
ADD COLUMN owner INTEGER REFERENCES users(id);
