SELECT e.id                                                                                            AS event_id,
       e.organization_id                                                                               AS organization_id,
       tt.id                                                                                           AS ticket_type_id,
       CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity))
                         FILTER (WHERE o.on_behalf_of_user_id IS NOT NULL),
                     0) AS BIGINT)                                                                     AS box_office_sales_in_cents,
       CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity))
                         FILTER (WHERE o.on_behalf_of_user_id IS NULL),
                     0) AS BIGINT)                                                                     AS online_sales_in_cents,
       CAST(COALESCE(SUM(oi.quantity) FILTER (WHERE o.on_behalf_of_user_id IS NOT NULL),
                     0) AS BIGINT)                                                                     AS box_office_count,
       CAST(COALESCE(SUM(oi.quantity) FILTER (WHERE o.on_behalf_of_user_id IS NULL), 0) AS BIGINT)     AS online_count,
       CAST(COALESCE(SUM((oi_t_fees.unit_price_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) +
                         (oi_e_fees.unit_price_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity)))
                         FILTER (WHERE o.on_behalf_of_user_id IS NOT NULL),
                     0) AS BIGINT)                                                                     AS total_box_office_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.unit_price_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) +
                         (oi_e_fees.unit_price_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity)))
                         FILTER (WHERE o.on_behalf_of_user_id IS NULL),
                     0) AS BIGINT)                                                                     AS total_online_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.company_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) +
                         (oi_e_fees.company_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity)))
                         FILTER (WHERE o.on_behalf_of_user_id IS NOT NULL),
                     0) AS BIGINT)                                                                     AS company_box_office_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.client_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) +
                         (oi_e_fees.client_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity)))
                         FILTER (WHERE o.on_behalf_of_user_id IS NOT NULL),
                     0) AS BIGINT)                                                                     AS client_box_office_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.company_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) +
                         (oi_e_fees.company_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity)))
                         FILTER (WHERE o.on_behalf_of_user_id IS NULL),
                     0) AS BIGINT)                                                                     AS company_online_fees_in_cents,
       CAST(COALESCE(SUM((oi_t_fees.client_fee_in_cents * (oi_t_fees.quantity - oi_t_fees.refunded_quantity)) +
                         (oi_e_fees.client_fee_in_cents * (oi_e_fees.quantity - oi_e_fees.refunded_quantity)))
                         FILTER (WHERE o.on_behalf_of_user_id IS NULL),
                     0) AS BIGINT)                                                                     AS client_online_fees_in_cents
FROM ticket_types tt
         LEFT JOIN events e ON tt.event_id = e.id
         LEFT JOIN order_items oi ON oi.ticket_type_id = tt.id AND oi.ticket_type_id IS NOT NULL
    --    -- Per ticket fees
         LEFT JOIN order_items oi_t_fees ON oi_t_fees.parent_id = oi.id AND oi_t_fees.item_type = 'PerUnitFees'
    --   -- Per event fees
         LEFT JOIN order_items oi_e_fees ON oi_e_fees.order_id = oi.order_id AND oi_e_fees.item_type = 'EventFees'
         RIGHT JOIN orders o ON oi.order_id = o.id AND o.status = 'Paid'
WHERE e.id IS NOT NULL
  AND ($1 IS NULL OR e.id = $1)
  AND ($2 IS NULL OR e.organization_id = $2)
GROUP BY e.organization_id, e.id, tt.id;
