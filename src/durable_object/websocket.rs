use worker::{Request, Response, Result, State, WebSocket, WebSocketPair};

pub async fn upgrade_websocket(req: Request, state: &State) -> Result<Response> {
    let page = req
        .url()?
        .query_pairs()
        .find(|(k, _)| k == "page")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default();

    let WebSocketPair { client, server } = WebSocketPair::new()?;
    state.accept_websocket_with_tags(&server, &[page.as_str()]);

    Response::from_websocket(client)
}

pub fn broadcast_count(state: &State, page: &str, total: u64, live_adjustment: i64) {
    let page_ws = state.get_websockets_with_tag(page);
    let live = (page_ws.len() as i64 + live_adjustment).max(0) as u64;

    let message = serde_json::json!({
        "page": page,
        "live": live,
        "total": total,
    })
    .to_string();

    for ws in &page_ws {
        ws.send_with_str(&message).ok();
    }
}

pub fn page_from_ws(state: &State, ws: &WebSocket) -> String {
    state.get_tags(ws).into_iter().next().unwrap_or_default()
}
