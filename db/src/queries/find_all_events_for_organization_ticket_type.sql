
SELECT t2.event_id,
       t2.Name,
       (SELECT min(tp.price_in_cents) FROM ticket_pricing tp WHERE tp.ticket_type_id = t2.id) AS min_price,
       (SELECT max(tp.price_in_cents) FROM ticket_pricing tp WHERE tp.ticket_type_id = t2.id) AS max_price,
       count(*)                                                                               AS total,
       CAST(sum(CASE
             WHEN ti.status in ( 'Purchased', 'Redeemed') AND ti.hold_id IS NULL THEN 1
             ELSE 0 END)     as BigInt)                                                                  AS sold_unreserved,
       CAST(sum(CASE
             WHEN ti.status IN ('Purchased', 'Redeemed') AND ti.hold_id IS NOT NULL THEN 1
             ELSE 0 END)   as BigInt)                                                                    AS sold_held,
       CAST(sum(CASE WHEN ti.status in ('Available', 'Reserved') AND ti.hold_id IS NULL THEN 1 ELSE 0 END)   as BigInt)                             AS open,
       CAST(sum(CASE WHEN ti.hold_id IS NOT NULL THEN 1 ELSE 0 END)   as BigInt)                             AS held,
       (SELECT cast(sum(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) as BIGINT)
        FROM order_items oi
               INNER JOIN orders o ON oi.order_id = o.id
        WHERE oi.ticket_type_id = t2.id
          AND o.status = 'Paid') as sales_total_in_cents
FROM ticket_instances ti
       INNER JOIN assets a
       INNER JOIN ticket_types t2
       INNER JOIN events e ON t2.event_id = e.id ON a.ticket_type_id = t2.id ON ti.asset_id = a.id
WHERE e.organization_id = $1
  AND CASE
        WHEN $2 IS NULL THEN TRUE -- All events
        WHEN $2 THEN e.event_start >= now() OR e.event_end > now() -- upcoming
        ELSE e.event_end <= now() END -- past
  AND ($3 IS NULL or e.id = $3)
  AND ti.status <> 'Nullified'
GROUP BY t2.name, t2.id, t2.event_id;
