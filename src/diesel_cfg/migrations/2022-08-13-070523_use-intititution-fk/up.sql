-- drop columns institution, campus from indentifications table
ALTER TABLE identifications DROP COLUMN IF EXISTS institution;
ALTER TABLE identifications DROP COLUMN IF EXISTS campus;

-- drop institution from profiles table
ALTER TABLE profiles DROP COLUMN IF EXISTS institution;

-- drop institution, campus from claimed_identifications
ALTER TABLE claimed_identifications DROP COLUMN IF EXISTS institution;
ALTER TABLE claimed_identifications DROP COLUMN IF EXISTS campus_location;


-- Add institution_id as foreign key to these tables
ALTER TABLE identifications
ADD COLUMN IF NOT EXISTS institution_id INTEGER REFERENCES institutions(id)
ON DELETE SET NULL;

ALTER TABLE claimed_identifications
ADD COLUMN IF NOT EXISTS institution_id INTEGER REFERENCES institutions(id)
ON DELETE SET NULL;

ALTER TABLE profiles
ADD COLUMN IF NOT EXISTS institution_id INTEGER REFERENCES institutions(id)
ON DELETE SET NULL;
