use worker::{Request, Response, Result, RouteContext};

use crate::api_worker::AppState;
use crate::api_worker::api::cors::CorsContext;

pub async fn public<F, Fut>(
    handler: F,
    req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response>
where
    F: Fn(Request, RouteContext<AppState>, CorsContext) -> Fut,
    Fut: std::future::Future<Output = Result<Response>>,
{
    let cors_context = CorsContext::new(ctx.data.config.allowed_origins.clone(), &req);
    if let Err(response) = cors_context.validate(&req) {
        return Ok(response);
    }
    handler(req, ctx, cors_context).await
}
