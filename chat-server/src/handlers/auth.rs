use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};

use crate::{models::CreateUser, AppError, AppState};

#[axum::debug_handler]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    Ok("Signin")
}

pub(crate) async fn signup_handler() -> impl IntoResponse {
    "signup"
}
