ALTER TABLE users
ADD COLUMN access_level INTEGER NOT NULL DEFAULT 2;

ALTER TABLE users
ADD CONSTRAINT access_level_range_check
CHECK(0 >= access_level AND access_level <= 2);
