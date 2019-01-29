SELECT a.ticket_type_id                                                                                                                      AS ticket_type_id,
       tt.name                                                                                                                               AS ticket_name,
       tt.status                                                                                                                             AS ticket_stats,
       e.id                                                                                                                                  AS event_id,
       e.name                                                                                                                                AS event_name,
       e.organization_id                                                                                                                     AS organization_id,

       -- Total Ticket Count
       CAST(COALESCE(COUNT(ti.id)  FILTER (WHERE ti.status != 'Nullified'), 0) AS BIGINT)                                                                                             AS allocation_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.status = 'Available'), 0) AS BIGINT)                                                      AS unpurchased_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NULL AND ti.status = 'Available'), 0) AS BIGINT)                               AS available_count,

       -------------------- COMPS --------------------
       -- Comp counts
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp'), 0) AS BIGINT)                              AS comp_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Available'), 0) AS BIGINT)  AS comp_available_count,
       -- comp_count - comp_available_count = the sum of these
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Redeemed'), 0) AS BIGINT)   AS comp_redeemed_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Purchased'), 0) AS BIGINT)  AS comp_purchased_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Reserved'), 0) AS BIGINT)   AS comp_reserved_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Nullified'), 0) AS BIGINT)  AS comp_nullified_count,
       ------------------ END COMPS ------------------

       -------------------- HOLDS --------------------
       -- Hold Counts
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp'), 0) AS BIGINT)                             AS hold_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Available'), 0) AS BIGINT) AS hold_available_count,
       -- hold_count - hold_available_count = the sum of these
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Redeemed'), 0) AS BIGINT)  AS hold_redeemed_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Purchased'), 0) AS BIGINT) AS hold_purchased_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Reserved'), 0) AS BIGINT)  AS hold_reserved_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Nullified'), 0) AS BIGINT) AS hold_nullified_count
       ------------------ END HOLDS -------------------

FROM ticket_instances ti
       LEFT JOIN assets a ON (a.id = ti.asset_id)
       LEFT JOIN ticket_types tt ON (tt.id = a.ticket_type_id)
       LEFT JOIN events e ON (e.id = tt.event_id)
       LEFT JOIN holds h ON (h.id = ti.hold_id)
WHERE ($1 IS NULL OR e.id = $1)
  AND ($2 IS NULL OR e.organization_id = $2)
GROUP BY a.ticket_type_id, tt.name, tt.status, e.id
HAVING COUNT(ti.id) FILTER (WHERE ti.status <> 'Nullified') > 1;
