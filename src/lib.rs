use axum::http::StatusCode;

pub mod models;
pub mod routes;
pub mod schema;

pub type HttpErr = (StatusCode, String);

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
pub fn internal_error<E>(err: E) -> HttpErr
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub fn bad_error<E>(err: E) -> HttpErr
where
    E: std::error::Error,
{
    (StatusCode::BAD_REQUEST, err.to_string())
}
