use crate::{models, schema, MainDatabase};
use super::{Error, Result};
use chrono::prelude::*;
use diesel::prelude::*;
use rocket_contrib::{json::Json, uuid::Uuid};

#[instrument(skip(conn))]
#[get("/handler")]
pub fn list(user: models::User, conn: MainDatabase) -> Result<Json<Vec<models::Handler>>> {
    use schema::handlers::dsl::*;

    Ok(Json(
        handlers
            .filter(user_id.eq(user.id))
            .load::<models::Handler>(&*conn)
            .map_err(Error::Database)?,
    ))
}

#[instrument(skip(conn))]
#[get("/handler/<uuid>")]
pub fn get(
    user: models::User,
    uuid: Uuid,
    conn: MainDatabase,
) -> Result<Json<models::Handler>> {
    use schema::handlers::dsl::*;
    let uuid = uuid.into_inner();
    let handler = handlers
        .find(uuid)
        .get_result::<models::Handler>(&*conn)
        .map_err(Error::Database)?;

    if handler.user_id != user.id {
        Err(Error::LackPermissions)
    } else {
        Ok(Json(handler))
    }
}

#[instrument(skip(conn))]
#[delete("/handler/<uuid>")]
pub fn delete(user: models::User, uuid: Uuid, conn: MainDatabase) -> Result {
    use schema::handlers::dsl::*;
    let uuid = uuid.into_inner();

    let hdl: models::Handler = handlers
        .find(uuid.clone())
        .get_result(&*conn)
        .map_err(Error::Database)?;

    if hdl.user_id != user.id && !user.is_admin {
        return Err(Error::LackPermissions);
    }

    diesel::update(handlers.find(uuid))
        .set(deleted_at.eq(Utc::now().naive_utc()))
        .get_result::<models::Handler>(&*conn)?;

    Ok(())
}
