drop operator class point_ops using btree;

drop function if exists point_lt cascade;
drop function if exists point_gt cascade;
drop function if exists point_lteq cascade;
drop function if exists point_gteq cascade;
drop function btpointcmp cascade;

drop operator = (point, point);
