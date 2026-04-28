use worker::{Request, Response, Result, State, WebSocketPair};

pub async fn upgrade_websocket(_req: Request, state: &State) -> Result<Response> {
    let WebSocketPair { client, server } = WebSocketPair::new()?;
    state.accept_web_socket(&server);

    Response::from_websocket(client)
}

pub fn broadcast_state(state: &State, total: u64) {
    let attached = state.get_websockets();
    broadcast(&attached, total, attached.len() as u64);
}

pub fn broadcast_after_close(state: &State, total: u64) {
    let attached = state.get_websockets();
    let live = (attached.len() as u64).saturating_sub(1);
    broadcast(&attached, total, live);
}

fn broadcast(attached: &[worker::WebSocket], total: u64, live: u64) {
    let message = serde_json::json!({
        "live": live,
        "total": total,
    })
    .to_string();

    for ws in attached {
        ws.send_with_str(&message).ok();
    }
}
