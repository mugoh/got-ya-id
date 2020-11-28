
--  Class Operator for the Point type
-- This requires superuser priviledges and can't
-- run on hosting services, since superusers aren't provided

/*
create operator = (leftarg = point, rightarg = point, procedure = point_eq, commutator = =);

create function point_lt(point, point)
returns boolean language sql immutable as $$
    select $1[0] < $2[0] or $1[0] = $2[0] and $1[1] < $2[1]
$$;

create operator < (leftarg = point, rightarg = point, procedure = point_lt, commutator = >);

create function point_gt(point, point)
returns boolean language sql immutable as $$
    select $1[0] > $2[0] or $1[0] = $2[0] and $1[1] > $2[1]
$$;

create operator > (leftarg = point, rightarg = point, procedure = point_gt, commutator = <);


create function point_gteq(point, point)
returns boolean language sql immutable as $$
    select ($1[0] > $2[0] or $1[0] = $2[0] and $1[1] > $2[1]) or ($1[0] = $2[0] and $1[1] = $2[1])
$$;

create operator >= (leftarg = point, rightarg = point, procedure = point_gteq, commutator = <=);


create function point_lteq(point, point)
returns boolean language sql immutable as $$
    select ($1[0] < $2[0] or $1[0] = $2[0] and $1[1] < $2[1]) or ($1[0] = $2[0] and $1[1] = $2[1])
$$;

create operator <= (leftarg = point, rightarg = point, procedure = point_lteq, commutator = >=);


create function btpointcmp(point, point)
returns integer language sql immutable as $$
    select case 
        when $1 = $2 then 0
        when $1 < $2 then -1
        else 1
    end
$$;


create operator class point_ops
    default for type point using btree as
        operator 1 <,
        operator 2 <=,
        operator 3 =,
        operator 4 >=,
        operator 5 >,
        function 1 btpointcmp(point, point);
        
*/
