SELECT DISTINCT oi.*
FROM order_items oi
LEFT JOIN holds h ON oi.hold_id = h.id
LEFT JOIN ticket_instances ti ON ti.order_item_id = oi.id
LEFT JOIN codes c ON oi.code_id = c.id
LEFT JOIN refunded_tickets rt ON oi.id = rt.order_item_id
LEFT JOIN (
    SELECT count(ti.id) as count, oi.id
    FROM order_items oi
    LEFT JOIN ticket_instances ti ON oi.id = ti.order_item_id
    WHERE oi.order_id = $1
    GROUP BY oi.id
) oit on oit.id = oi.id
WHERE oi.order_id = $1
AND item_type = 'Tickets'
AND (
    ti.status = 'Nullified'
    OR ti.reserved_until < now()
    OR c.end_date < now()
    OR h.end_at < now()
    OR oit.count <> oi.quantity
)
