-- $1 : query
-- $2 : start date
-- $3 : end date
-- $4 : user id
-- $5 : bool(true if user has admin role)
-- $6 : region id
-- $7 : list of statuses

select distinct e.id, e.name, e.organization_id, e.venue_id, e.created_at, e.event_start,
e.door_time, e.status, e.publish_date, e.promo_image_url, e.additional_info, e.age_limit, e.cancelled_at,
e.updated_at
from events e
  left join event_artists ea
    inner join artists a
      on ea.artist_id = a.id
     -- and a.name ilike '%' ||COALESCE($1, '') ||'%'
    on e.id= ea.event_id
  inner join organizations o
    left join organization_users ou
      on o.id = ou.organization_id
    on e.organization_id = o.id
  left join venues v
    on e.venue_id = v.id;
-- where ($2 is null or e.event_start > $2) and ($3 is null or e.event_start < $3)
-- and (e.name ilike '%' || COALESCE($1, '') ||'%' or v.name ilike '%' || COALESCE($1, '') ||'%')
-- and ($5 = 1 or ou.user_id = $4 or o.owner_user_id = $4)
-- and ($6 is null or $6 = v.region_id)
-- and ($7 is null or e.status = any($7) )
--order by e.event_start asc, e.name asc;