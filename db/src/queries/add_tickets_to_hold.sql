UPDATE ticket_instances
SET
    hold_id   = $1,
    updated_at = now()
WHERE id IN (SELECT t.id
             FROM ticket_instances AS t
                    INNER JOIN assets AS a ON t.asset_id = a.id
             WHERE ((t.reserved_until < now() AND t.status = 'Reserved') OR t.status = 'Available')
               AND a.ticket_type_id = $2
               AND coalesce($4, 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11') =
                   coalesce(t.hold_id, 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11') -- dummy guid
             LIMIT $3 FOR UPDATE SKIP LOCKED)
    RETURNING
      id,
      asset_id,
      token_id,
      hold_id,
      order_item_id,
      wallet_id,
      reserved_until,
      status,
      redeem_key,
      transfer_key,
      transfer_expiry_date,
      created_at,
      updated_at;
