ALTER TABLE identifications DROP COLUMN IF EXISTS institution_id;

ALTER TABLE identifications
ADD COLUMN IF NOT EXISTS institution VARCHAR;

ALTER TABLE identifications
ADD COLUMN IF NOT EXISTS campus VARCHAR;

--
ALTER TABLE claimed_identifications DROP COLUMN IF EXISTS institution_id;
ALTER TABLE claimed_identifications

ADD COLUMN IF NOT EXISTS institution VARCHAR;
ALTER TABLE claimed_identifications
ADD COLUMN IF NOT EXISTS campus_location VARCHAR;

--

ALTER TABLE profiles DROP COLUMN IF EXISTS institution_id;
ALTER TABLE profiles
ADD COLUMN IF NOT EXISTS institution VARCHAR;
