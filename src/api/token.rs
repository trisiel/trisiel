use super::{Error, Result};
use crate::{jwt, models, schema, MainDatabase};
use chrono::prelude::*;
use diesel::prelude::*;
use rocket_contrib::{json::Json, uuid::Uuid};

#[instrument(skip(conn), err)]
#[get("/token")]
pub fn list(user: models::User, conn: MainDatabase) -> Result<Json<Vec<models::Token>>> {
    use schema::tokens::dsl::*;

    Ok(Json(
        tokens
            .filter(user_id.eq(user.id))
            .load::<models::Token>(&*conn)
            .map_err(Error::Database)?,
    ))
}

#[instrument(skip(conn), err)]
#[delete("/token/<uuid>")]
pub fn delete(user: models::User, conn: MainDatabase, uuid: Uuid) -> Result {
    use schema::tokens::dsl::*;
    let uuid = uuid.into_inner();

    let tok: models::Token = tokens
        .find(uuid.clone())
        .get_result(&*conn)
        .map_err(Error::Database)?;

    if tok.user_id != user.id && !user.is_admin {
        return Err(Error::LackPermissions);
    }

    diesel::update(tokens.find(uuid))
        .set(deleted_at.eq(Utc::now().naive_utc()))
        .get_result::<models::Token>(&*conn)?;

    Ok(())
}

#[instrument(skip(conn), err)]
#[post("/token")]
pub fn create(user: models::User, conn: MainDatabase) -> Result<String> {
    use schema::tokens;

    let tok: models::Token = diesel::insert_into(tokens::table)
        .values(&models::NewToken {
            user_id: user.id.clone(),
        })
        .get_result(&*conn)
        .map_err(Error::Database)?;

    Ok(jwt::make(user.id, tok.id)?)
}
