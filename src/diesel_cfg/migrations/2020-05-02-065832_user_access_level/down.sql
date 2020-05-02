ALTER TABLE users  DROP COLUMN access_level;
ALTER TABLE users DROP CONSTRAINT access_level_range_check;
