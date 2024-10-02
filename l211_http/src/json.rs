use crate::error::AppError;
use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct SucceedResponse<T>
where
    T: Serialize,
{
    result: T,
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    T: Serialize,
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(SucceedResponse { result: self.0 }).into_response()
    }
}
