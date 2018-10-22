use chrono::prelude::*;
use diesel;
use diesel::dsl::{exists, select};
use diesel::expression::dsl;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable};
use models::*;
use schema::{order_items, orders};
use serde_json;
use std::collections::HashMap;
use time::Duration;
use utils::errors;
use utils::errors::*;
use uuid::Uuid;

#[derive(Associations, Debug, Identifiable, PartialEq, Queryable)]
#[belongs_to(User)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    order_type: String,
    order_date: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "orders"]
pub struct NewOrder {
    user_id: Uuid,
    status: String,
    expires_at: NaiveDateTime,
    order_type: String,
}

impl NewOrder {
    pub fn commit(&self, conn: &PgConnection) -> Result<Order, DatabaseError> {
        use schema::orders;
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new order",
            diesel::insert_into(orders::table)
                .values(self)
                .get_result(conn),
        )
    }
}

impl Order {
    pub fn create(user_id: Uuid, order_type: OrderTypes) -> NewOrder {
        NewOrder {
            user_id,
            status: OrderStatus::Draft.to_string(),
            expires_at: Utc::now().naive_utc() + Duration::minutes(15),
            order_type: order_type.to_string(),
        }
    }

    pub fn destroy(&self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Failed to destroy order record",
            diesel::delete(self).execute(conn),
        )
    }

    pub fn status(&self) -> OrderStatus {
        self.status.parse::<OrderStatus>().unwrap()
    }

    pub fn find_cart_for_user(user_id: Uuid, conn: &PgConnection) -> Result<Order, DatabaseError> {
        orders::table
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.eq("Draft"))
            .filter(orders::order_type.eq("Cart"))
            .filter(orders::expires_at.ge(dsl::now))
            .first(conn)
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not load cart for user",
            )
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Order, DatabaseError> {
        orders::table
            .filter(orders::id.eq(id))
            .first(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not find order")
    }

    pub fn add_tickets(
        &self,
        ticket_type_id: Uuid,
        hold_redemption_key: Option<String>,
        quantity: i64,
        conn: &PgConnection,
    ) -> Result<Vec<TicketInstance>, DatabaseError> {
        let ticket_pricing = TicketPricing::get_current_ticket_pricing(ticket_type_id, conn)?;
        let ticket_type = TicketType::find(ticket_type_id, conn)?;
        let hold_id = if hold_redemption_key.is_some() {
            Some(Hold::find_by_redemption_key(hold_redemption_key, conn)?.id);
        } else {
            None;
        };

        let event = Event::find(ticket_type.event_id, conn)?;
        let organization = Organization::find(event.organization_id, conn)?;

        let fee_schedule_range = FeeSchedule::find(organization.fee_schedule_id, conn)?
            .get_range(ticket_pricing.price_in_cents, conn)?
            .unwrap();

        let order_item = match OrderItem::find_for_ticket_pricing(self.id, ticket_pricing.id, conn)
            .optional()?
        {
            Some(mut o) => {
                o.quantity = o.quantity + quantity;
                o.update(conn)?;
                o
            }
            None => NewTicketsOrderItem {
                order_id: self.id,
                item_type: OrderItemTypes::Tickets.to_string(),
                quantity,
                ticket_pricing_id: ticket_pricing.id,
                event_id: Some(event.id),
                fee_schedule_range_id: fee_schedule_range.id,
                unit_price_in_cents: ticket_pricing.price_in_cents,
            }.commit(conn)?,
        };

        order_item.update_fees(conn)?;
        self.update_event_fees(conn)?;
        TicketInstance::reserve_tickets(
            &order_item,
            &self.expires_at,
            ticket_type_id,
            hold_id,
            quantity,
            conn,
        )
    }

    pub fn has_items(&self, conn: &PgConnection) -> Result<bool, DatabaseError> {
        select(exists(
            order_items::table.filter(order_items::order_id.eq(self.id)),
        )).get_result(conn)
        .to_db_error(
            errors::ErrorCode::QueryError,
            "Could not check if order items exist",
        )
    }

    pub fn remove_tickets(
        &self,
        mut order_item: OrderItem,
        quantity: Option<i64>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        TicketInstance::release_tickets(&order_item, quantity, conn)?;
        let calculated_quantity = order_item.calculate_quantity(conn)?;

        if calculated_quantity == 0 {
            order_item.destroy(conn)?;
        } else {
            let ticket_pricing = TicketPricing::find(order_item.ticket_pricing_id.unwrap(), conn)?;
            order_item.quantity = calculated_quantity;
            order_item.unit_price_in_cents = ticket_pricing.price_in_cents;
            order_item.update(conn)?;

            order_item.update_fees(conn)?;
        }

        self.update_event_fees(conn)?;

        Ok(())
    }

    pub fn update_event_fees(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let order_items = OrderItem::find_for_order(self.id, conn)?;
        let mut order_items_per_event: HashMap<Uuid, Vec<OrderItem>> = HashMap::new();

        for o in order_items {
            if o.event_id.is_some() {
                order_items_per_event
                    .entry(o.event_id.unwrap())
                    .or_insert_with(|| Vec::new())
                    .push(o);
            }
        }
        for k in order_items_per_event.keys() {
            let mut has_event_fee = false;
            let event = Event::find(*k, conn)?;
            let mut item_count = 0;
            for o in order_items_per_event.get(k).unwrap() {
                item_count += 1;
                if o.item_type == OrderItemTypes::EventFees.to_string() {
                    has_event_fee = true;
                }
            }
            //If there is an event fee but it is the only order_item left for this event then
            //delete the event fee.
            if has_event_fee && item_count == 1 {
                let event_fee = &order_items_per_event.get(k).unwrap()[0];
                OrderItem::find(event_fee.id, conn)?.destroy(conn)?;
            } else if !has_event_fee {
                let mut new_event_fee = NewFeesOrderItem {
                    order_id: self.id,
                    item_type: OrderItemTypes::EventFees.to_string(),
                    event_id: Some(event.id),
                    unit_price_in_cents: 0,
                    quantity: 1,
                    parent_id: None,
                };
                let organization = Organization::find(event.organization_id, conn)?;
                if event.fee_in_cents.is_some() {
                    new_event_fee.unit_price_in_cents = event.fee_in_cents.unwrap();
                    new_event_fee.commit(conn)?;
                } else if organization.event_fee_in_cents.is_some() {
                    new_event_fee.unit_price_in_cents = organization.event_fee_in_cents.unwrap();
                    new_event_fee.commit(conn)?;
                }
            }
        }

        Ok(())
    }

    pub fn find_for_user_for_display(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrder>, DatabaseError> {
        use schema::*;
        let orders: Vec<Order> = orders::table
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.ne(OrderStatus::Draft.to_string()))
            .order_by(orders::order_date.desc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load orders")?;
        let mut r = Vec::<DisplayOrder>::new();
        for order in orders {
            r.push(order.for_display(conn)?);
        }
        Ok(r)
    }

    pub fn items(&self, conn: &PgConnection) -> Result<Vec<OrderItem>, DatabaseError> {
        OrderItem::find_for_order(self.id, conn)
    }

    pub fn for_display(&self, conn: &PgConnection) -> Result<DisplayOrder, DatabaseError> {
        let now = Utc::now().naive_utc();
        let seconds_until_expiry = if self.expires_at >= now {
            let duration = self.expires_at.signed_duration_since(now);
            duration.num_seconds() as u32
        } else {
            0
        };

        Ok(DisplayOrder {
            id: self.id,
            status: self.status.clone(),
            date: self.order_date,
            expires_at: self.expires_at,
            items: self.items_for_display(conn)?,
            total_in_cents: self.calculate_total(conn)?,
            seconds_until_expiry,
        })
    }

    pub fn items_for_display(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrderItem>, DatabaseError> {
        OrderItem::find_for_display(self.id, conn)
    }

    pub fn find_item(
        &self,
        cart_item_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrderItem, DatabaseError> {
        OrderItem::find_in_order(self.id, cart_item_id, conn)
    }

    pub fn add_external_payment(
        &mut self,
        external_reference: String,
        current_user_id: Uuid,
        amount: i64,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        let payment = Payment::create(
            self.id,
            current_user_id,
            PaymentStatus::Completed,
            PaymentMethods::External,
            "External".to_string(),
            external_reference,
            amount,
            None,
        );
        self.add_payment(payment, conn)
    }

    pub fn add_credit_card_payment(
        &mut self,
        current_user_id: Uuid,
        amount: i64,
        provider: String,
        external_reference: String,
        status: PaymentStatus,
        provider_data: serde_json::Value,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        let payment = Payment::create(
            self.id,
            current_user_id,
            status,
            PaymentMethods::CreditCard,
            provider,
            external_reference,
            amount,
            Some(provider_data),
        );

        self.add_payment(payment, conn)
    }

    fn add_payment(
        &mut self,
        payment: NewPayment,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        if self.status() == OrderStatus::Paid {
            return DatabaseError::business_process_error("This order has already been paid");
        }
        // orders can only expire if the order is in draft
        if self.status() == OrderStatus::Draft {
            self.mark_partially_paid(conn)?;
        } else if self.status() != OrderStatus::PartiallyPaid {
            return DatabaseError::business_process_error(&format!(
                "Order was in unexpected state when trying to make a payment: {}",
                self.status()
            ));
        }

        let p = payment.commit(conn)?;

        self.complete_if_fully_paid(conn)?;
        Ok(p)
    }

    fn mark_partially_paid(&mut self, conn: &PgConnection) -> Result<(), DatabaseError> {
        // TODO: The multiple queries in this method could probably be combined into a single query
        let now_plus_one_day = Utc::now().naive_utc() + Duration::days(1);

        let result = diesel::update(
            orders::table.filter(
                orders::id
                    .eq(self.id)
                    .and(orders::updated_at.eq(self.updated_at))
                    .and(orders::expires_at.gt(dsl::now)),
            ),
        ).set((
            orders::status.eq(OrderStatus::PartiallyPaid.to_string()),
            orders::expires_at.eq(now_plus_one_day),
            orders::updated_at.eq(dsl::now),
        )).execute(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not update order status")?;

        let db_record = Order::find(self.id, conn)?;

        if result == 0 {
            if db_record.updated_at != self.updated_at {
                return DatabaseError::concurrency_error(
                    "Could not update order because it has been updated by another process",
                );
            }

            // Unfortunately, it's quite hard to work out what the current dsl::now() time is
            // So assume that the order has expired.
            return DatabaseError::business_process_error(
                "Could not update order because it has expired",
            );
        }

        self.updated_at = db_record.updated_at;
        self.status = db_record.status;

        //Extend the reserved_until time for tickets associated with this order
        let order_items = OrderItem::find_for_order(db_record.id, conn)?;

        for item in &order_items {
            TicketInstance::update_reserved_time(item, now_plus_one_day, conn)?;
        }

        Ok(())
    }

    pub(crate) fn complete_if_fully_paid(
        &mut self,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        if self.total_paid(conn)? >= self.calculate_total(conn)? {
            self.update_status(OrderStatus::Paid, conn)?;
            //Mark tickets as Purchased
            let order_items = OrderItem::find_for_order(self.id, conn)?;
            for item in &order_items {
                TicketInstance::mark_as_purchased(item, self.user_id, conn)?;
            }
        }
        Ok(())
    }

    pub fn total_paid(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        #[derive(QueryableByName)]
        struct ResultForSum {
            #[sql_type = "Nullable<BigInt>"]
            s: Option<i64>,
        };
        let query = diesel::sql_query(
            "SELECT CAST(SUM(amount) as BigInt) as s FROM payments WHERE order_id = $1 AND status='Completed';",
        ).bind::<diesel::sql_types::Uuid, _>(self.id);

        let sum: ResultForSum = query.get_result(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not get total payments for order",
        )?;
        Ok(sum.s.unwrap_or(0))
    }

    fn update_status(
        &mut self,
        status: OrderStatus,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        self.status = status.to_string();
        diesel::update(&*self)
            .set((
                orders::status.eq(&self.status),
                orders::updated_at.eq(dsl::now),
            )).execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update order")?;

        Ok(())
    }

    pub fn calculate_total(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        let order_items = self.items(conn)?;
        let mut total = 0;

        for item in &order_items {
            total += item.unit_price_in_cents * item.quantity;
        }

        Ok(total)
    }
}

#[derive(Deserialize, Serialize)]
pub struct DisplayOrder {
    pub id: Uuid,
    pub date: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub seconds_until_expiry: u32,
    pub status: String,
    pub items: Vec<DisplayOrderItem>,
    pub total_in_cents: i64,
}
