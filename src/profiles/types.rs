use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::entity::{Profile, UserRole};

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum UserRoleType {
    Patient,
    Doctor,
    Admin,
}

impl From<UserRole> for UserRoleType {
    fn from(r: UserRole) -> Self {
        match r {
            UserRole::Patient => UserRoleType::Patient,
            UserRole::Doctor => UserRoleType::Doctor,
            UserRole::Admin => UserRoleType::Admin,
        }
    }
}

impl From<UserRoleType> for UserRole {
    fn from(r: UserRoleType) -> Self {
        match r {
            UserRoleType::Patient => UserRole::Patient,
            UserRoleType::Doctor => UserRole::Doctor,
            UserRoleType::Admin => UserRole::Admin,
        }
    }
}

#[derive(SimpleObject)]
pub struct ProfileType {
    pub id: Uuid,
    pub auth_user_id: Uuid,
    pub email: String,
    pub locale: String,
    pub role: UserRoleType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<Profile> for ProfileType {
    fn from(p: Profile) -> Self {
        Self {
            id: p.id,
            auth_user_id: p.auth_user_id,
            email: p.email,
            locale: p.locale,
            role: p.role.into(),
            created_at: p.created_at,
            updated_at: p.updated_at,
            deleted_at: p.deleted_at,
        }
    }
}

#[derive(InputObject)]
pub struct SyncProfileInput {
    pub role: UserRoleType,
}

#[derive(InputObject)]
pub struct UpdateProfileInput {
    pub locale: Option<String>,
}
