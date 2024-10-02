use crate::repo::InMemoryStorageError;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'static str>,
}

#[derive(Debug)]
pub enum AppError {
    ServiceUnavailable(&'static str),
    JsonRejection(JsonRejection),
    _Internal(anyhow::Error),
    _InMemoryStorage(InMemoryStorageError),
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        use AppError::*;

        let (status, error) = match self {
            JsonRejection(_rejection) => (StatusCode::BAD_REQUEST, None),
            ServiceUnavailable(why) => (StatusCode::SERVICE_UNAVAILABLE, Some(why)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        };

        (status, Json(ErrorResponse { error })).into_response()
    }
}
