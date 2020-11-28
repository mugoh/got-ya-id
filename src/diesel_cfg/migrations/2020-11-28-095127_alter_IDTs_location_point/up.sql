ALTER TABLE identifications
DROP COLUMN location_point,
ADD COLUMN location_latitude FLOAT,
ADD COLUMN location_longitude FLOAT;
