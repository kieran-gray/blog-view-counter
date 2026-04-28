pub mod storage;
pub mod websocket;

use std::cell::Cell;

use tracing::{debug, error};
use worker::{
    DurableObject, Env, Request, Response, Result, SqlStorage, State, WebSocket,
    WebSocketIncomingMessage, durable_object,
};

use crate::durable_object::{
    storage::{flush_count, init_schema, load_count},
    websocket::{broadcast_after_close, broadcast_state, upgrade_websocket},
};

#[durable_object]
pub struct ViewCounter {
    state: State,
    cached_total: Cell<Option<u64>>,
}

impl ViewCounter {
    fn sql(&self) -> SqlStorage {
        self.state.storage().sql()
    }

    fn total_views(&self) -> Result<u64> {
        if let Some(total) = self.cached_total.get() {
            return Ok(total);
        }
        let sql = self.sql();
        init_schema(&sql)?;
        let total = load_count(&sql)?;
        self.cached_total.set(Some(total));
        Ok(total)
    }
}

impl DurableObject for ViewCounter {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            cached_total: Cell::new(None),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if req.path() != "/websocket" {
            return Response::error("Not Found", 404);
        }

        let new_total = self.total_views()? + 1;

        let response = upgrade_websocket(req, &self.state).await?;
        self.cached_total.set(Some(new_total));

        broadcast_state(&self.state, new_total);

        self.state
            .storage()
            .set_alarm(std::time::Duration::from_secs(5))
            .await?;

        Ok(response)
    }

    async fn websocket_message(
        &self,
        _ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        if matches!(&message, WebSocketIncomingMessage::String(s) if s == "{}") {
            match self.total_views() {
                Ok(total) => broadcast_state(&self.state, total),
                Err(e) => error!(error = %e, "Failed to load total views"),
            }
        }
        Ok(())
    }

    async fn websocket_close(
        &self,
        _ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        match self.total_views() {
            Ok(total) => broadcast_after_close(&self.state, total),
            Err(e) => error!(error = %e, "Failed to load total views"),
        }
        Ok(())
    }

    async fn alarm(&self) -> Result<Response> {
        debug!("Alarm triggered, persisting count");
        let total = self.total_views()?;
        if total == 0 {
            return Response::empty();
        }
        if let Err(e) = flush_count(&self.sql(), total) {
            error!(error = %e, "Failed to persist view count to SQL");
        }
        Response::empty()
    }
}
