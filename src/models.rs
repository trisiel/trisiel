use serde::Serialize;
use uuid::Uuid;
use crate::schema::{gitea_tokens, users};

#[derive(Insertable, Queryable, Serialize, Debug, Clone)]
#[table_name="users"]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub salutation: String,
    pub is_admin: bool,
    pub is_locked: bool,
    pub tier: i32,
}

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name="gitea_tokens"]
pub struct GiteaToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}
