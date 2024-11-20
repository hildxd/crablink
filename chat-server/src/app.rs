use std::{ops::Deref, sync::Arc};

use axum::{routing::post, Router};

use crate::{
    handlers::{signin_handler, signup_handler},
    AppConfig,
};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);

    let api = Router::new()
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler));

    Router::new().nest("/api", api).with_state(state)
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub(crate) fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}
