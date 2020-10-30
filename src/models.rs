use crate::schema::*;
use chrono::NaiveDateTime;
use serde::Serialize;
use std::fmt;
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

#[derive(Queryable, Serialize, Clone)]
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

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Insertable)]
#[table_name = "gitea_tokens"]
pub struct NewGiteaToken {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Queryable, Debug, Clone)]
pub struct GiteaToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "tokens"]
pub struct NewToken {
    pub user_id: Uuid,
}

#[derive(Queryable, Debug, Clone, Serialize)]
pub struct Token {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "handlers"]
pub struct NewHandler {
    pub user_id: Uuid,
    pub human_name: String,
    pub current_version: Option<String>,
    pub async_impl: bool,
}

#[derive(Queryable, Debug, Clone, Serialize)]
pub struct Handler {
    pub id: Uuid,
    pub user_id: Uuid,
    pub human_name: String,
    pub current_version: Option<String>,
    pub async_impl: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "handler_config"]
pub struct NewHandlerConfig {
    pub key_name: String,
    pub value_contents: String,
    pub handler_id: Uuid,
}

#[derive(Queryable, Debug, Clone, Serialize)]
pub struct HandlerConfig {
    pub key_name: String,
    pub value_contents: String,
    pub handler_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
