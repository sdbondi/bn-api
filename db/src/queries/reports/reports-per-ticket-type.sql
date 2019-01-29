SELECT a.ticket_type_id                                                                                                                                                                                                                                        AS ticket_type_id,
       tt.name                                                                                                                                                                                                                                                 AS ticket_name,
       tt.status                                                                                                                                                                                                                                               AS ticket_stats,
       e.id                                                                                                                                                                                                                                                    AS event_id,
       e.name                                                                                                                                                                                                                                                  AS event_name,
       e.organization_id                                                                                                                                                                                                                                       AS organization_id,

       -- Total Ticket Count
       CAST(COALESCE(COUNT(ti.id), 0) AS BIGINT)                                                                                                                                                                                                               AS allocation_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.status = 'Available'), 0) AS BIGINT)                                                                                                                                                                        AS unallocated_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.status = 'Reserved'), 0) AS BIGINT)                                                                                                                                                                         AS reserved_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.status = 'Redeemed'), 0) AS BIGINT)                                                                                                                                                                         AS redeemed_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.status = 'Purchased'), 0) AS BIGINT)                                                                                                                                                                        AS purchased_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.status = 'Nullified'), 0) AS BIGINT)                                                                                                                                                                        AS nullified_count,
       -- Not in a hold and not purchased / reserved / redeemed etc
       -- What can a generic user purchase.
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NULL AND ti.status = 'Available'), 0) AS BIGINT)                                                                                                                                                 AS available_for_purchase_count,

       -------------------- COMPS --------------------
       -- Comp counts
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp'), 0) AS BIGINT)                                                                                                                                                AS comp_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Available'), 0) AS BIGINT)                                                                                                                    AS comp_available_count,
       -- comp_count - comp_available_count = the sum of these
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Redeemed'), 0) AS BIGINT)                                                                                                                     AS comp_redeemed_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Purchased'), 0) AS BIGINT)                                                                                                                    AS comp_purchased_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Reserved'), 0) AS BIGINT)                                                                                                                     AS comp_reserved_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type = 'Comp' AND ti.status = 'Nullified'), 0) AS BIGINT)                                                                                                                    AS comp_nullified_count,
       ------------------ END COMPS ------------------

       -------------------- HOLDS --------------------
       -- Hold Counts
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp'), 0) AS BIGINT)                                                                                                                                               AS hold_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Available'), 0) AS BIGINT)                                                                                                                   AS hold_available_count,
       -- hold_count - hold_available_count = the sum of these
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Redeemed'), 0) AS BIGINT)                                                                                                                    AS hold_redeemed_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Purchased'), 0) AS BIGINT)                                                                                                                   AS hold_purchased_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Reserved'), 0) AS BIGINT)                                                                                                                    AS hold_reserved_count,
       CAST(COALESCE(COUNT(ti.id) FILTER (WHERE ti.hold_id IS NOT NULL AND h.hold_type != 'Comp' AND ti.status = 'Nullified'), 0) AS BIGINT)                                                                                                                   AS hold_nullified_count,
       ------------------ END HOLDS -------------------

       -- Actual Values
       CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) FILTER (WHERE p.is_box_office IS TRUE AND o.status = 'Paid'), 0) AS BIGINT)                                                                                            AS box_office_sales_in_cents,
       CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) FILTER (WHERE p.is_box_office IS FALSE AND o.status = 'Paid'), 0) AS BIGINT)                                                                                           AS online_sales_in_cents,


       --Total Sold Count
       CAST(COALESCE(SUM(oi.quantity) FILTER (WHERE p.is_box_office IS TRUE AND o.status = 'Paid' AND (h.hold_type IS NULL OR h.hold_type != 'Comp')), 0) AS BIGINT)                                                                                           AS box_office_sale_count,
       CAST(COALESCE(SUM(oi.quantity) FILTER (WHERE p.is_box_office IS FALSE AND o.status = 'Paid' AND (h.hold_type IS NULL OR h.hold_type != 'Comp')), 0) AS BIGINT)                                                                                          AS online_sale_count,
       CAST(COALESCE(SUM(oi.quantity) FILTER (WHERE h.hold_type = 'Comp' AND o.status = 'Paid'), 0) AS BIGINT)                                                                                                                                                 AS comp_sale_count,

       --Refunded
       --We can use AVG here because we aggregate the values in the sub-query below so we only return a single value
       CAST(AVG(oi_refunds.total_refunded_quantity) AS BIGINT)                                                                                                                                                                                                 AS total_refunded_count,

       CAST(COALESCE(SUM((oi_t_fees.unit_price_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) + (oi_e_fees.unit_price_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity))) FILTER (WHERE p.is_box_office IS TRUE), 0) AS BIGINT)    AS total_box_office_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.unit_price_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) + (oi_e_fees.unit_price_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity))) FILTER (WHERE p.is_box_office IS FALSE), 0) AS BIGINT)   AS total_online_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.company_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) + (oi_e_fees.company_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity))) FILTER (WHERE p.is_box_office IS TRUE),
                     0) AS BIGINT)                                                                                                                                                                                                                             AS company_box_office_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.client_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) + (oi_e_fees.client_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity))) FILTER (WHERE p.is_box_office IS TRUE), 0) AS BIGINT)    AS client_box_office_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.company_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) + (oi_e_fees.company_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity))) FILTER (WHERE p.is_box_office IS FALSE), 0) AS BIGINT) AS company_online_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.client_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) + (oi_e_fees.client_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity))) FILTER (WHERE p.is_box_office IS FALSE), 0) AS BIGINT)   AS client_online_fees_in_cents

FROM ticket_instances ti
       LEFT JOIN assets a ON (a.id = ti.asset_id)
       LEFT JOIN ticket_types tt ON (tt.id = a.ticket_type_id)
       LEFT JOIN events e ON (e.id = tt.event_id)
       LEFT JOIN holds h ON (h.id = ti.hold_id)
       LEFT JOIN order_items oi ON (oi.id = ti.order_item_id)
       LEFT JOIN ticket_pricing tp ON oi.ticket_pricing_id = tp.id
       LEFT JOIN orders o ON (o.id = oi.order_id)
       LEFT JOIN (SELECT order_id, CAST(ARRAY_TO_STRING(ARRAY_AGG(DISTINCT p.payment_method), ', ') LIKE '%External' AS BOOLEAN) AS is_box_office FROM payments p GROUP BY p.payment_method, p.order_id) AS p on o.id = p.order_id
       LEFT JOIN (SELECT ticket_type_id, SUM(refunded_quantity) AS total_refunded_quantity FROM order_items oi_refunds WHERE oi_refunds.ticket_type_id IS NOT NULL GROUP BY oi_refunds.ticket_type_id) AS oi_refunds ON oi.ticket_type_id = oi_refunds.ticket_type_id
  -- Per ticket fees
       LEFT JOIN order_items oi_t_fees ON oi_t_fees.parent_id = oi.id AND oi_t_fees.item_type = 'PerUnitFees'
  -- Per event fees
       LEFT JOIN order_items oi_e_fees ON oi_e_fees.order_id = oi.order_id AND oi_e_fees.item_type = 'EventFees'
WHERE ($1 IS NULL OR e.id = $1)
  AND ($2 IS NULL OR e.organization_id = $2)
  AND ($3 IS NULL OR o.paid_at >= $3)
  AND ($4 IS NULL OR o.paid_at <= $4)
GROUP BY a.ticket_type_id, tt.name, tt.status, e.id;