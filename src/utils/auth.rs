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


