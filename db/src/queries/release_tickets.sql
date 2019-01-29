UPDATE ticket_instances
SET
    order_item_id = NULL,
    reserved_until = NULL,
    redeem_key = NULL,
    transfer_key = NULL,
    transfer_expiry_date = NULL,
    status = $5,
    updated_at = now()
WHERE id IN (SELECT t.id
             FROM ticket_instances AS t
                    INNER JOIN assets AS a ON t.asset_id = a.id
             WHERE t.order_item_id = $1
             AND t.status = $3
             AND t.id = COALESCE($4, t.id)
             LIMIT $2 FOR UPDATE SKIP LOCKED)
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
