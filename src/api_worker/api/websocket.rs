use tracing::error;
use worker::{Request, Response, RouteContext};

use crate::api_worker::{AppState, api::cors::CorsContext};

pub async fn handle_websocket_connect(
    req: Request,
    ctx: RouteContext<AppState>,
    cors_context: CorsContext,
) -> worker::Result<Response> {
    let page = match ctx.param("page") {
        Some(p) => p.to_string(),
        None => {
            let error = "No page provided";
            error!(error);
            return Ok(cors_context.add_to_response(Response::error(error, 400)?));
        }
    };

    if !ctx.data.config.allowed_paths.contains(&page) {
        let error = format!("Path not allowed: {page}");
        error!(error);
        return Ok(cors_context.add_to_response(Response::error(error, 403)?));
    }

    match req.headers().get("Upgrade") {
        Ok(Some(h)) if h == "websocket" => (),
        _ => return Ok(Response::empty()?.with_status(426)),
    }

    let response = ctx
        .data
        .do_client
        .websocket_upgrade(&page)
        .await
        .map_err(|e| format!("Failed to connect to durable object: {e}"))?;

    Ok(cors_context.add_to_response(response))
}
