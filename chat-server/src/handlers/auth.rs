use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrorOutput,
    models::{CreateUser, User, VerifyUser},
    AppError, AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

#[allow(unused_variables)]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<VerifyUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;

    match user {
        Some(user) => {
            let token = state.sk.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.pool).await?;
    let token = state.sk.sign(user.clone())?;
    Ok((StatusCode::CREATED, Json(AuthOutput { token })).into_response())
}

#[cfg(test)]
mod tests {
    use crate::{utils::parser_response, AppConfig};

    use super::*;

    #[tokio::test]
    async fn test_signup_success() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("hildxd", "hildxd@qq.com", "password");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::CREATED);
        let ret = parser_response::<AuthOutput>(ret).await?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_signin_success() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("hildxd", "hildxd@qq.com", "password");
        User::create(&input, &state.pool).await?;

        let input = VerifyUser::new("hildxd@qq.com", "password");
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let ret = parser_response::<AuthOutput>(ret).await?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_signin_fails_with_wrong_password() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("hildxd", "hildxd@qq.com", "password");
        User::create(&input, &state.pool).await?;

        let input = VerifyUser::new("hildxd@qq.com", "wrong_password");
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);
        Ok(())
    }

    #[tokio::test]
    async fn test_signup_fails_with_duplicate_email() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("hildxd", "email@qq.com", "password");
        signup_handler(State(state.clone()), Json(input.clone())).await?;

        let ret = signup_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let ret = parser_response::<ErrorOutput>(ret).await?;
        assert_eq!(ret.error, "email is exist: email@qq.com");
        Ok(())
    }

    #[tokio::test]
    async fn test_signin_fails_with_nonexistent_email() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("hildxd", "email@qq.com", "password");
        signup_handler(State(state.clone()), Json(input)).await?;

        let input = VerifyUser::new("xxemail@qq.com", "password");
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);
        Ok(())
    }
}
