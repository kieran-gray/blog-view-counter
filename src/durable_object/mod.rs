pub mod storage;
pub mod websocket;

use std::cell::{Cell, RefCell};

use tracing::{debug, error};
use worker::{
    DurableObject, Env, Request, Response, Result, State, WebSocket, WebSocketIncomingMessage,
    durable_object,
};

use crate::durable_object::{
    storage::{flush_count, init_schema, load_count},
    websocket::{broadcast_count, upgrade_websocket},
};

#[durable_object]
pub struct ViewCounter {
    state: State,
    _env: Env,
    initialized: Cell<bool>,
    count: RefCell<u64>,
}

impl ViewCounter {
    fn ensure_initialized(&self) {
        if !self.initialized.get() {
            let sql = self.state.storage().sql();
            match init_schema(&sql).and_then(|_| load_count(&sql)) {
                Ok(loaded) => {
                    *self.count.borrow_mut() = loaded;
                    self.initialized.set(true);
                }
                Err(e) => error!(error = %e, "Failed to load view counts from storage"),
            }
        }
    }
}

impl DurableObject for ViewCounter {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            _env: env,
            initialized: Cell::new(false),
            count: RefCell::new(0),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        self.ensure_initialized();

        if req.path() == "/websocket" {
            let response = upgrade_websocket(req, &self.state).await?;

            let total = {
                let mut count = self.count.borrow_mut();
                *count += 1;
                *count
            };

            broadcast_count(&self.state, total, 0);
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(5))
                .await?;

            return Ok(response);
        }
        Response::error("Not Found", 404)
    }

    async fn websocket_message(
        &self,
        _ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        match message {
            WebSocketIncomingMessage::String(s) if s == "{}" => {
                self.ensure_initialized();
                let total = *self.count.borrow();
                broadcast_count(&self.state, total, 0);
            }
            _ => {}
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
        self.ensure_initialized();
        let total = *self.count.borrow();
        broadcast_count(&self.state, total, -1);
        Ok(())
    }

    async fn alarm(&self) -> Result<Response> {
        debug!("Alarm triggered, persisting counts");
        self.ensure_initialized();
        let counts = self.count.borrow();
        if *counts == 0 {
            return Response::empty();
        }

        let sql = self.state.storage().sql();
        if let Err(e) = flush_count(&sql, *counts) {
            error!(error = %e, "Failed to persist view counts to SQL");
        }

        Response::empty()
    }
}
