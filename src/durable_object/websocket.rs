use worker::{Request, Response, Result, State, WebSocketPair};

pub async fn upgrade_websocket(_req: Request, state: &State) -> Result<Response> {
    let WebSocketPair { client, server } = WebSocketPair::new()?;
    state.accept_web_socket(&server);

    Response::from_websocket(client)
}

pub fn broadcast_count(state: &State, total: u64, live_adjustment: i64) {
    let ws = state.get_websockets();
    let live = (ws.len() as i64 + live_adjustment).max(0) as u64;

    let message = serde_json::json!({
        "live": live,
        "total": total,
    })
    .to_string();

    for ws in &ws {
        ws.send_with_str(&message).ok();
    }
}
