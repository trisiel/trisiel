use crate::schema::{gitea_tokens, users};
use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub salutation: String,
    pub is_admin: bool,
    pub is_locked: bool,
    pub tier: i32,
}

#[derive(Queryable, Serialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub salutation: String,
    pub is_admin: bool,
    pub is_locked: bool,
    pub tier: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name="gitea_tokens"]
pub struct NewGiteaToken {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "gitea_tokens"]
pub struct GiteaToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
