use worker::Router;

use crate::api_worker::AppState;
use crate::api_worker::api::middleware::public;
use crate::api_worker::api::websocket::handle_websocket_connect;

pub fn create_router(app_state: AppState) -> Router<'static, AppState> {
    Router::with_data(app_state).on_async("/api/v1/connect/:page", |req, ctx| {
        public(handle_websocket_connect, req, ctx)
    })
}
