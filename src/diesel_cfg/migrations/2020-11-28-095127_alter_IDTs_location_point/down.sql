-- This file should undo anything in `up.sql`


drop operator if exists class point_ops using btree;

drop function if exists point_lt cascade;
drop function if exists point_gt cascade;
drop function if exists point_lteq cascade;
drop function if exists point_gteq cascade;
drop function btpointcmp cascade;

drop operator = (point, point);



ALTER TABLE identifications
ADD COLUMN IF NOT EXISTS location_point
DROP COLUMN location_latitude FLOAT
DROP COLUMN location_longitude FLOAT;
