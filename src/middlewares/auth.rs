use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::DecodingKey;
use tracing::error;

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let decoding_key = req
        .extensions()
        .get::<DecodingKey>()
        .cloned()
        .ok_or_else(|| {
            error!("❌ No Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("❌ No Authorization header");
            StatusCode::UNAUTHORIZED
        })?;
    
    Ok(next.run(req).await)
}
