SELECT event_id,
       ticket_type_id,
       ticket_pricing_id,
       total_sold,
       comp_count,
       box_office_count,
       online_count,
       price_in_cents,
       total_company_fee_in_cents,
       total_client_fee_in_cents,
       pricing_name,
       ticket_name,
       CAST(total_net_income + total_company_fee_in_cents +
            total_client_fee_in_cents AS BIGINT) AS total_gross_income_in_cents
FROM (
         SELECT oi.event_id,
                oi.ticket_type_id,
                oi.ticket_pricing_id,
                CAST(SUM(oi.quantity - oi.refunded_quantity) AS BIGINT)        AS total_sold,
                CAST(
                    COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE h.hold_type = 'Comp'),
                             0) AS BIGINT)                                     AS comp_count,
                CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE orders.box_office_pricing = TRUE),
                              0) AS BIGINT)                                    AS box_office_count,
                CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE orders.box_office_pricing = FALSE),
                              0) AS BIGINT)                                    AS online_count,
                CAST(AVG(tp.price_in_cents) AS BIGINT)                         AS price_in_cents, -- face price
                CAST(COALESCE(SUM((oi.quantity - oi.refunded_quantity) * oi_fees.company_fee_in_cents), 0) AS BIGINT) AS total_company_fee_in_cents,
                CAST(COALESCE(SUM((oi.quantity - oi.refunded_quantity) * oi_fees.client_fee_in_cents), 0) AS BIGINT)  AS total_client_fee_in_cents,

                CAST(COALESCE(SUM((oi.quantity - oi.refunded_quantity) * oi.unit_price_in_cents), 0) AS BIGINT)       AS total_net_income,
                tp.name                                                        AS pricing_name,
                tt.name                                                        AS ticket_name
         FROM orders
                  LEFT JOIN order_items oi ON orders.id = oi.order_id
                  LEFT JOIN order_items oi_fees ON oi.id = oi_fees.parent_id
                  LEFT JOIN ticket_types tt ON (oi.ticket_type_id = tt.id)
                  LEFT JOIN ticket_pricing tp ON (oi.ticket_pricing_id = tp.id)
                  LEFT JOIN holds h ON oi.hold_id = h.id
                  LEFT JOIN events e ON oi.event_id = e.id
         WHERE orders.status = 'Paid'
           AND ($1 IS NULL OR oi.event_id = $1)
           AND ($2 IS NULL OR e.organization_id = $2)
           AND oi.item_type = 'Tickets'
           AND ($3 IS NULL OR orders.paid_at >= $3)
           AND ($4 IS NULL OR orders.paid_at <= $4)
         GROUP BY oi.event_id, oi.ticket_type_id, oi.ticket_pricing_id, tt.name, tp.name
     ) AS report_data;
