use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::entity::User;

#[derive(SimpleObject)]
pub struct UserType {
    pub id: Uuid,
    pub supabase_uid: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserType {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            supabase_uid: u.supabase_uid,
            email: u.email,
            username: u.username,
            avatar_url: u.avatar_url,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}
