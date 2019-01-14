use actix_web::HttpResponse;
use actix_web::Path;
use bigneon_db::prelude::*;
use db::Connection;
use errors::BigNeonError;
use models::*;

#[derive(Deserialize)]
pub struct PathParameters {
    code: String,
}

pub fn show(
    (connection, path): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    let hold = Hold::find_by_redemption_code(&path.code, conn)?;

    #[derive(Serialize)]
    struct R {
        ticket_type: UserDisplayTicketType,
        redemption_code: String,
        max_per_order: Option<i64>,
        discount_in_cents: Option<i64>,
        hold_type: HoldTypes,
    }

    let redemption_code = hold.redemption_code.clone();
    let max_per_order = hold.max_per_order;
    let discount_in_cents = hold.discount_in_cents;
    let hold_type = hold.hold_type;

    let ticket_type = UserDisplayTicketType::from_ticket_type_and_hold(
        &TicketType::find(hold.ticket_type_id, conn)?,
        &FeeSchedule::find(
            Organization::find_for_event(hold.event_id, conn)?.fee_schedule_id,
            conn,
        )?,
        false,
        Some(hold),
        conn,
    )?;

    let r = R {
        ticket_type,
        redemption_code,
        max_per_order,
        discount_in_cents,
        hold_type,
    };
    return Ok(HttpResponse::Ok().json(r));
}
