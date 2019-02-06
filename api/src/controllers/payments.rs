use crate::db::Connection;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Path;
use actix_web::Query;
use actix_web::State;
use bigneon_db::prelude::*;
use errors::*;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct QueryParams {
    success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PathParams {
    id: Uuid,
    nonce: String,
}

pub fn callback(
    (query, path, connection, state, request): (
        Query<QueryParams>,
        Path<PathParams>,
        Connection,
        State<AppState>,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    let mut order = Order::find(path.id, conn)?;
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
        order.reset_to_draft(None, conn)?;
        application::redirect(&format!(
            "{}/events/{}/tickets/confirmation",
            state.config.front_end_url,
            order.main_event_id(conn)?
        ))
    }
}
