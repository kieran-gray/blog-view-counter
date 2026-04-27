pub mod storage;
pub mod websocket;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use tracing::{debug, error};
use worker::{
    DurableObject, Env, Request, Response, Result, State, WebSocket, WebSocketIncomingMessage,
    durable_object,
};

use crate::durable_object::{
    storage::{flush_counts, init_schema, load_counts},
    websocket::{broadcast_count, page_from_ws, upgrade_websocket},
};

#[durable_object]
pub struct ViewCounter {
    state: State,
    _env: Env,
    initialized: Cell<bool>,
    counts: RefCell<HashMap<String, u64>>,
}

impl DurableObject for ViewCounter {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            _env: env,
            initialized: Cell::new(false),
            counts: RefCell::new(HashMap::new()),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if req.path() == "/websocket" {
            return upgrade_websocket(req, &self.state).await;
        }
        Response::error("Not Found", 404)
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        match message {
            WebSocketIncomingMessage::String(s) if s == "{}" => {}
            _ => return Ok(()),
        }

        let page = page_from_ws(&self.state, &ws);
        if page.is_empty() {
            return Ok(());
        }

        if !self.initialized.get() {
            let sql = self.state.storage().sql();
            match init_schema(&sql).and_then(|_| load_counts(&sql)) {
                Ok(loaded) => *self.counts.borrow_mut() = loaded,
                Err(e) => error!(error = %e, "Failed to load view counts from storage"),
            }
            self.initialized.set(true);
        }

        let total = {
            let mut counts = self.counts.borrow_mut();
            let entry = counts.entry(page.clone()).or_insert(0);
            *entry += 1;
            *entry
        };

        broadcast_count(&self.state, &page, total, 0);

        self.state
            .storage()
            .set_alarm(std::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        let page = page_from_ws(&self.state, &ws);
        if !page.is_empty() {
            let total = self.counts.borrow().get(&page).copied().unwrap_or(0);
            broadcast_count(&self.state, &page, total, -1);
        }
        Ok(())
    }

    async fn alarm(&self) -> Result<Response> {
        debug!("Alarm triggered, persisting counts");
        let counts = self.counts.borrow();
        if counts.is_empty() {
            return Response::empty();
        }

        let sql = self.state.storage().sql();
        if let Err(e) = init_schema(&sql).and_then(|_| flush_counts(&sql, &counts)) {
            error!(error = %e, "Failed to persist view counts to SQL");
        }

        Response::empty()
    }
}
