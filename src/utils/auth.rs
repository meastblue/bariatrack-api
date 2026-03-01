use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn supabase_uid(&self) -> Result<Uuid, uuid::Error> {
        self.sub.parse()
    }
}

// ── JWKS types ─────────────────────────────

#[derive(Debug, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
pub struct JwkKey {
    pub kty: String,
    pub crv: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub alg: Option<String>,
}

/// Fetch the JWKS from Supabase and build a DecodingKey for ES256
pub async fn fetch_jwks_key(supabase_url: &str) -> Result<DecodingKey, String> {
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url);

    let resp = reqwest::get(&jwks_url)
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {e}"))?;

    let jwks: JwksResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {e}"))?;

    let key = jwks
        .keys
        .iter()
        .find(|k| k.kty == "EC" && k.crv.as_deref() == Some("P-256"))
        .ok_or("No EC P-256 key found in JWKS")?;

    let x = key.x.as_ref().ok_or("Missing x in JWK")?;
    let y = key.y.as_ref().ok_or("Missing y in JWK")?;

    DecodingKey::from_ec_components(x, y).map_err(|e| format!("Failed to build key: {e}"))
}

pub fn validate_token(token: &str, decoding_key: &DecodingKey) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&["authenticated"]);

    decode::<Claims>(token, decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {e}"))
}

pub fn extract_bearer_token(header: &str) -> Option<&str> {
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
}
