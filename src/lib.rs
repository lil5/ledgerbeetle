pub mod models;
pub mod routes;
pub mod schema;

pub mod http_err {
    use axum::http::StatusCode;
    pub type HttpErr = (StatusCode, String);
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
}
