CREATE OR REPLACE FUNCTION order_items_code_id_max_uses_valid(UUID, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
      (select max_uses from codes where id = $2) >
      (
          select count(distinct orders.id)
          from orders
          join order_items
          on order_items.order_id = orders.id
          where orders.id <> $1 and order_items.code_id = $2
      )
    );
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION order_items_ticket_type_id_valid_for_access_code(UUID, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
      -- If there's no access code associated or the code passed in is valid
      SELECT ttc.id is null or ttc2.id is not null
      FROM ticket_types tt
      LEFT JOIN (
          -- Associated codes which grant access
          SELECT ttc.ticket_type_id, c.id
          FROM ticket_type_codes ttc
          JOIN codes c ON ttc.code_id = c.id
          JOIN ticket_types tt ON tt.id = ttc.ticket_type_id
          WHERE c.code_type = 'Access' AND tt.id = $1
      ) ttc ON ttc.ticket_type_id = tt.id
      LEFT JOIN ticket_type_codes ttc2
      ON ttc2.ticket_type_id = tt.id
      AND ttc2.code_id = $2
      AND ttc2.code_id = ttc.id
      WHERE tt.id = $1
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE order_items ADD CONSTRAINT order_items_ticket_type_id_valid_for_access_code CHECK(order_items_ticket_type_id_valid_for_access_code(ticket_type_id, code_id));
