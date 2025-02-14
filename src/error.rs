use anyhow::Error;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

pub enum AxumError {
    Internal(Error),
    NotFound(String),
}

impl IntoResponse for AxumError {
    fn into_response(self) -> Response {
        match self {
            AxumError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("internal error: {:?}", err),
            )
                .into_response(),
            AxumError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
        }
    }
}

impl<E> From<E> for AxumError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AxumError::Internal(err.into())
    }
}
