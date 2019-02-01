use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::orders::Order;
use models::*;
use schema::payments;
use serde_json;
use utils::errors::*;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Identifiable, PartialEq, Queryable)]
pub struct Payment {
    pub id: Uuid,
    order_id: Uuid,
    created_by: Option<Uuid>,
    pub status: PaymentStatus,
    pub payment_method: PaymentMethods,
    pub amount: i64,
    pub provider: String,
    pub external_reference: Option<String>,
    raw_data: Option<serde_json::Value>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Payment {
    pub(crate) fn create(
        order_id: Uuid,
        created_by: Option<Uuid>,
        status: PaymentStatus,
        payment_method: PaymentMethods,
        provider: String,
        external_reference: Option<String>,
        amount: i64,
        raw_data: Option<serde_json::Value>,
    ) -> NewPayment {
        NewPayment {
            order_id,
            created_by,
            status,
            payment_method,
            provider,
            external_reference,
            amount,
            raw_data,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Payment, DatabaseError> {
        payments::table
            .filter(payments::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find payment")
    }

    pub fn find_by_order(
        order_id: Uuid,
        external_reference: &str,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        payments::table
            .filter(
                payments::order_id
                    .eq(order_id)
                    .and(payments::external_reference.eq(external_reference)),
            )
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find payment for order")
            .expect_single()
    }

    pub fn log_refund(
        &self,
        current_user_id: Uuid,
        refund_amount: u32,
        refund_data: Option<serde_json::Value>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        Payment::create(
            self.order_id,
            self.created_by,
            PaymentStatus::Refunded,
            self.payment_method,
            self.provider.clone(),
            self.external_reference.clone(),
            -(refund_amount as i64),
            refund_data.clone(),
        )
        .commit(Some(current_user_id), conn)?;

        DomainEvent::create(
            DomainEventTypes::PaymentRefund,
            "Payment was refunded".to_string(),
            Tables::Payments,
            Some(self.id),
            Some(current_user_id),
            refund_data,
        )
        .commit(conn)?;
        Ok(())
    }

    pub fn add_ipn(
        &self,
        new_status: PaymentStatus,
        raw_data: serde_json::Value,
        current_user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        self.update_status(new_status, current_user_id, conn)?;
        DomainEvent::create(
            DomainEventTypes::PaymentProviderIPN,
            "Payment IPN received".to_string(),
            Tables::Payments,
            Some(self.id),
            current_user_id,
            Some(raw_data),
        )
        .commit(conn)?;

        Ok(())
    }

    fn update_status(
        &self,
        status: PaymentStatus,
        current_user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        diesel::update(payments::table.filter(payments::id.eq(self.id)))
            .set((
                payments::status.eq(status),
                payments::updated_at.eq(dsl::now),
            ))
            .execute(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not change the status of payment",
            )?;

        DomainEvent::create(
            DomainEventTypes::PaymentUpdated,
            format!("Payment status updated to {}", status),
            Tables::Payments,
            Some(self.id),
            current_user_id,
            Some(json!({ "new_status": status })),
        )
        .commit(conn)?;

        Ok(())
    }

    pub fn update_amount(
        &self,
        current_user_id: Option<Uuid>,
        amount: i64,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let old_amount = self.amount;

        diesel::update(self)
            .set((
                payments::amount.eq(&amount),
                payments::updated_at.eq(dsl::now),
            ))
            .execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update amount on payment")?;
        DomainEvent::create(
            DomainEventTypes::PaymentUpdated,
            "Payment Amount was updated".to_string(),
            Tables::Payments,
            Some(self.id),
            current_user_id,
            Some(json!({
            "old_amount": old_amount, "new_amount": amount
            })),
        )
        .commit(conn)?;

        Ok(())
    }

    pub fn mark_complete(
        &self,
        raw_data: serde_json::Value,
        current_user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        if self.status == PaymentStatus::Completed {
            DomainEvent::create(
                DomainEventTypes::PaymentCompleted,
                "Payment was completed".to_string(),
                Tables::Payments,
                Some(self.id),
                current_user_id,
                Some(raw_data),
            )
            .commit(conn)?;
            return Ok(());
        }
        self.update_status(PaymentStatus::Completed, current_user_id, conn)?;

        DomainEvent::create(
            DomainEventTypes::PaymentCompleted,
            "Payment was completed".to_string(),
            Tables::Payments,
            Some(self.id),
            current_user_id,
            Some(raw_data),
        )
        .commit(conn)?;

        self.order(conn)?
            .complete_if_fully_paid(current_user_id, conn)?;

        Ok(())
    }

    fn order(&self, conn: &PgConnection) -> Result<Order, DatabaseError> {
        use schema::*;
        orders::table
            .find(self.order_id)
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve order")
    }
}

#[derive(Insertable)]
#[table_name = "payments"]
pub struct NewPayment {
    order_id: Uuid,
    created_by: Option<Uuid>,
    pub status: PaymentStatus,
    payment_method: PaymentMethods,
    external_reference: Option<String>,
    amount: i64,
    provider: String,
    raw_data: Option<serde_json::Value>,
}

impl NewPayment {
    pub(crate) fn commit(
        self,
        current_user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        let res: Payment = diesel::insert_into(payments::table)
            .values(&self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create payment")?;

        DomainEvent::create(
            DomainEventTypes::PaymentCreated,
            "Payment created".to_string(),
            Tables::Payments,
            Some(res.id),
            current_user_id,
            self.raw_data,
        )
        .commit(conn)?;
        Ok(res)
    }
}
