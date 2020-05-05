ALTER TABLE users
ADD COLUMN access_level INTEGER NOT NULL DEFAULT 2;

ALTER TABLE users
ADD CONSTRAINT access_level_range_check
CHECK(access_level >= 0 AND access_level <= 2);
