use serde::Serialize;
use uuid::Uuid;
use crate::schema::users;

#[derive(Insertable, Queryable, Serialize, Debug, Clone)]
#[table_name="users"]
pub struct User {
    id: Uuid,
    email: String,
    salutation: String,
    is_admin: bool,
    is_locked: bool,
    tier: i32,
}
