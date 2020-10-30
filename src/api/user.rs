use crate::models;
use super::{Error, Result};
use rocket_contrib::{json::Json, uuid::Uuid};

#[instrument]
#[get("/user/<uuid>")]
pub fn get(user: models::User, uuid: Uuid) -> Result<Json<models::User>> {
    if uuid != user.id {
        return Err(Error::LackPermissions);
    }

    Ok(Json(user))
}

#[instrument]
#[get("/whoami")]
pub fn whoami(user: models::User) -> Json<models::User> {
    Json(user)
}
