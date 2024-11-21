use std::{ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use axum::{routing::post, Router};
use sqlx::PgPool;

use crate::{
    handlers::{signin_handler, signup_handler},
    utils::{DecodingKey, EncodingKey},
    AppConfig, AppError,
};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) pk: DecodingKey,
    pub(crate) sk: EncodingKey,
    pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::new(config).await?;

    let api = Router::new()
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler));

    let app = Router::new().nest("/api", api).with_state(state);
    Ok(app)
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub(crate) async fn new(config: AppConfig) -> Result<Self, AppError> {
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                pk,
                sk,
                pool,
            }),
        })
    }
}

#[cfg(test)]
impl AppState {
    pub(crate) async fn new_for_test(
        config: AppConfig,
    ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let tdb = sqlx_db_tester::TestPg::new(
            config.server.db_url.clone(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let state = Self {
            inner: Arc::new(AppStateInner {
                config,
                pk,
                sk,
                pool,
            }),
        };
        Ok((tdb, state))
    }
}
