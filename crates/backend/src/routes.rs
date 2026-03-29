mod health;
mod files;
mod convert;
mod preview;

use axum::Router;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .merge(health::routes())
        .merge(files::routes())
        .merge(convert::routes())
        .merge(preview::routes())
        .with_state(state)
}
