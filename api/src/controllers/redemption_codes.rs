use actix_web::HttpResponse;
use actix_web::Path;
use bigneon_db::prelude::*;
use db::Connection;
use errors::BigNeonError;
use uuid::Uuid;

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
        ticket_type_id: Uuid,
        redemption_code: String,
        max_per_order: Option<i64>,
        discount_in_cents: Option<i64>,
        hold_type: HoldTypes,
    }
    let r = R {
        ticket_type_id: hold.ticket_type_id,
        redemption_code: hold.redemption_code.clone(),
        max_per_order: hold.max_per_order,
        discount_in_cents: hold.discount_in_cents,
        hold_type: hold.hold_type,
    };
    return Ok(HttpResponse::Ok().json(r));
}
