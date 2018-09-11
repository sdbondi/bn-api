use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use schema::{fee_schedule_ranges, fee_schedules};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Queryable, Identifiable, Clone)]
pub struct FeeSchedule {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl FeeSchedule {
    pub fn create(name: String, ranges: Vec<(i64, i64)>) -> NewFeeSchedule {
        NewFeeSchedule { name, ranges }
    }
    pub fn ranges(&self, conn: &PgConnection) -> Result<Vec<FeeScheduleRange>, DatabaseError> {
        fee_schedule_ranges::table
            .filter(fee_schedule_ranges::fee_schedule_id.eq(self.id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load fee schedule ranges")
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<FeeSchedule, DatabaseError> {
        fee_schedules::table
            .find(id)
            .first::<FeeSchedule>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading Fee Schedule")
    }
}

pub struct NewFeeSchedule {
    name: String,
    ranges: Vec<(i64, i64)>,
}

impl NewFeeSchedule {
    pub fn commit(self, conn: &PgConnection) -> Result<FeeSchedule, DatabaseError> {
        let result: FeeSchedule = diesel::insert_into(fee_schedules::table)
            .values(fee_schedules::name.eq(&self.name))
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create fee schedule")?;

        for range in &self.ranges {
            diesel::insert_into(fee_schedule_ranges::table)
                .values((
                    fee_schedule_ranges::fee_schedule_id.eq(result.id),
                    fee_schedule_ranges::min_price.eq(range.0),
                    fee_schedule_ranges::fee.eq(range.1),
                ))
                .execute(conn)
                .to_db_error(
                    ErrorCode::InsertError,
                    "Could not create fee schedule range",
                )?;
        }

        Ok(result)
    }
}

#[derive(Queryable, Serialize)]
pub struct FeeScheduleRange {
    #[allow(dead_code)]
    id: Uuid,
    #[allow(dead_code)]
    fee_schedule_id: Uuid,
    pub min_price: i64,
    pub fee: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}