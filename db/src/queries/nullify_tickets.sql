UPDATE ticket_instances
SET
  status = 'Nullified'
WHERE id IN (SELECT t.id
             FROM ticket_instances AS t
             WHERE t.asset_id = $1 AND t.status in ('Available', 'Reserved') AND t.hold_id IS NULL
             -- Nullify Available inventory prior to Reserved
             ORDER BY t.status, t.reserved_until
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
