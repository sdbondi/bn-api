use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{OrderStatus, User};
use schema::orders;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(User)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    status: String,
    #[allow(dead_code)]
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "orders"]
pub struct NewOrder {
    user_id: Uuid,
    status: String,
}

impl NewOrder {
    pub fn commit(&self, conn: &Connectable) -> Result<Order, DatabaseError> {
        use schema::orders;
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new order",
            diesel::insert_into(orders::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl Order {
    pub fn create(user_id: Uuid) -> NewOrder {
        NewOrder {
            user_id,
            status: OrderStatus::Unpaid.to_string(),
        }
    }
    pub fn status(&self) -> OrderStatus {
        return OrderStatus::parse(&self.status).unwrap();
    }
}
