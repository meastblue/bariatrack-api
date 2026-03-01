use crate::utils::auth::{extract_bearer_token, validate_token};
use axum::{body::Body, extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::DecodingKey;
use tracing::{info, error};

pub async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let decoding_key = request
        .extensions()
        .get::<DecodingKey>()
        .cloned()
        .ok_or_else(|| {
            error!("❌ DecodingKey not found in extensions");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("❌ No Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    let token = extract_bearer_token(auth_header).ok_or_else(|| {
        error!("❌ Could not extract Bearer token");
        StatusCode::UNAUTHORIZED
    })?;

    let claims = validate_token(token, &decoding_key).map_err(|e| {
        error!("❌ Token validation failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    info!("✅ Authenticated: {}", claims.sub);

    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
