use axum::http::StatusCode;

pub type HttpResult<T> = Result<T, HttpErr>;
pub type HttpErr = (StatusCode, String);
pub fn internal_error<E>(err: E) -> HttpErr
where
    E: ToString,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub fn bad_error<E>(err: E) -> HttpErr
where
    E: ToString + std::fmt::Debug + std::fmt::Display,
{
    (StatusCode::BAD_REQUEST, err.to_string())
}

pub fn teapot_error<E>(err: E) -> HttpErr
where
    E: ToString + std::fmt::Debug + std::fmt::Display,
{
    (StatusCode::IM_A_TEAPOT, err.to_string())
}
