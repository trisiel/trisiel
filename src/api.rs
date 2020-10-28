use crate::{schema, models, MainDatabase};
use diesel::prelude::*;
use rocket_contrib::{json::Json, uuid::Uuid};

#[tracing::instrument(skip(conn))]
#[get("/user/<uuid>")]
pub fn get_user(conn: MainDatabase, uuid: Uuid) -> Json<models::User> {
    use schema::users::dsl::users;
    let result = users
        .find(uuid.into_inner())
        .get_result::<models::User>(&*conn)
        .expect("to find user");

    Json(result)
}
