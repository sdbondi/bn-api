use crate::db::Connection;
use actix_web::HttpResponse;
use actix_web::Path;
use actix_web::Query;
use actix_web::State;
use bigneon_db::prelude::*;
use errors::*;
use extractors::OptionalUser;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryParams {
    pub success: bool,
}

// The nonce is in the path so that it is not cached.
#[derive(Serialize, Deserialize, Debug)]
pub struct PathParams {
    pub nonce: String,
    pub id: Uuid,
}

pub fn callback(
    (query, path, connection, state, user): (
        Query<QueryParams>,
        Path<PathParams>,
        Connection,
        State<AppState>,
        OptionalUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    let order = Order::find(path.id, conn)?;
    let mut payments: Vec<Payment> = order
        .payments(conn)?
        .into_iter()
        .filter(|p| p.url_nonce.as_ref() == Some(&path.nonce))
        .collect();

    let payment = payments.pop();
    let payment = match payment {
        Some(p) => p,
        None => return application::not_found(),
    };

    // We specifically don't count this as a payment confirmation, that will be done via the IPN
    // Just redirect to page accordingly

    if query.success {
        if payment.status == PaymentStatus::Requested {
            payment.mark_pending_ipn(user.id(), conn)?;
        }
        application::redirect(&format!(
            "{}/events/{}/tickets/success",
            state.config.front_end_url,
            order.main_event_id(conn)?
        ))
    } else {
        payment.mark_cancelled(
            json!({"path": &path.into_inner(), "query": &query.into_inner()}),
            None,
            conn,
        )?;
        // order.reset_to_draft(None, conn)?;
        application::redirect(&format!(
            "{}/events/{}/tickets/confirmation",
            state.config.front_end_url,
            order.main_event_id(conn)?
        ))
    }
}
