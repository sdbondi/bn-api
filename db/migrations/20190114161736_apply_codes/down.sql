CREATE OR REPLACE FUNCTION order_items_code_id_max_uses_valid(UUID, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
      (select max_uses from codes where id = $2) >
      (
          select count(orders.id)
          from orders
          join order_items
          on order_items.order_id = orders.id
          where orders.id <> $1 and order_items.code_id = $2
          group by orders.id
      )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE order_items DROP CONSTRAINT order_items_ticket_type_id_valid_for_access_code;
DROP FUNCTION order_items_ticket_type_id_valid_for_access_code;
