use super::{Error, Result};
use crate::models;
use rocket_contrib::{json::Json, uuid::Uuid};

#[get("/user/<uuid>")]
#[instrument(err)]
pub fn get(user: models::User, uuid: Uuid) -> Result<Json<models::User>> {
    if uuid != user.id {
        return Err(Error::LackPermissions);
    }

    Ok(Json(user))
}

#[get("/whoami")]
#[instrument]
pub fn whoami(user: models::User) -> Json<models::User> {
    Json(user)
}
