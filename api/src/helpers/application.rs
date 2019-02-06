use actix_web::{http, http::StatusCode, HttpResponse, Responder};
use auth::user::User as AuthUser;
use errors::*;
use serde_json::{self, Value};
use std::collections::HashMap;

pub fn unauthorized<T: Responder>(
    user: Option<AuthUser>,
    additional_data: Option<HashMap<&'static str, Value>>,
) -> Result<T, BigNeonError> {
    unauthorized_with_message(
        "User does not have the required permissions",
        user,
        additional_data,
    )
}

pub fn unauthorized_with_message<T: Responder>(
    message: &str,
    auth_user: Option<AuthUser>,
    additional_data: Option<HashMap<&'static str, Value>>,
) -> Result<T, BigNeonError> {
    if let Some(auth_user) = auth_user {
        auth_user.log_unauthorized_access_attempt(additional_data.unwrap_or(HashMap::new()));
    }

    Err(AuthError::new(AuthErrorType::Unauthorized, message.into()).into())
}

pub fn forbidden<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    Err(AuthError::new(AuthErrorType::Forbidden, message.into()).into())
}

pub fn unprocessable<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    Err(
        ApplicationError::new_with_type(ApplicationErrorType::Unprocessable, message.to_string())
            .into(),
    )
}

pub fn internal_server_error<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    error!("Internal Server Error: {}", message);
    Err(ApplicationError::new(message.to_string()).into())
}

pub fn no_content() -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::new(StatusCode::NO_CONTENT))
}

pub fn not_found() -> Result<HttpResponse, BigNeonError> {
    warn!("Not found");
    Ok(HttpResponse::new(StatusCode::NOT_FOUND))
}

pub fn created(json: serde_json::Value) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::new(StatusCode::CREATED)
        .into_builder()
        .json(json))
}

pub fn redirect(url: &str) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, url)
        .finish())
}
