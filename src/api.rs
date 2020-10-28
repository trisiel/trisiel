use crate::{jwt, models, schema, MainDatabase};
use chrono::prelude::*;
use color_eyre::eyre::Report;
use diesel::prelude::*;
use rocket::http::{ContentType, Status};
use rocket::request::{self, FromRequest, Request};
use rocket::response::Responder;
use rocket::Outcome;
use rocket::Response;
use rocket_contrib::{json::Json, uuid::Uuid};
use std::io::Cursor;

#[tracing::instrument]
#[get("/user/<uuid>")]
pub fn get_user(user: models::User, uuid: Uuid) -> Result<Json<models::User>> {
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

#[instrument(skip(conn))]
#[get("/token")]
pub fn get_tokens(user: models::User, conn: MainDatabase) -> Result<Json<Vec<models::Token>>> {
    use schema::tokens::dsl::*;

    Ok(Json(
        tokens
            .filter(user_id.eq(user.id))
            .load::<models::Token>(&*conn)
            .map_err(Error::Database)?,
    ))
}

#[instrument(skip(conn))]
#[delete("/token/<uuid>")]
pub fn delete_token(user: models::User, conn: MainDatabase, uuid: Uuid) -> Result {
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

#[instrument(skip(conn))]
#[post("/token")]
pub fn create_token(user: models::User, conn: MainDatabase) -> Result<String> {
    use schema::tokens;

    let tok: models::Token = diesel::insert_into(tokens::table)
        .values(&models::NewToken {
            user_id: user.id.clone(),
        })
        .get_result(&*conn).map_err(Error::Database)?;

    Ok(jwt::make(user.id, tok.id)?)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("bad or no authorization")]
    BadOrNoAuth,

    #[error("you lack needed permissions")]
    LackPermissions,

    #[error("internal server error: {0}")]
    InternalServerError(#[from] Report),

    #[error("external dependency failed: {0}")]
    ExternalDependencyFailed(Report),
}

impl<'a> Responder<'a> for Error {
    fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'a>, Status> {
        match self {
            Error::Database(why) => Response::build()
                .header(ContentType::Plain)
                .status(Status::InternalServerError)
                .sized_body(Cursor::new(format!("{}", why)))
                .ok(),
            Error::BadOrNoAuth | Error::LackPermissions => Response::build()
                .header(ContentType::Plain)
                .status(Status::Unauthorized)
                .sized_body(Cursor::new(format!("{}", self)))
                .ok(),
            Error::InternalServerError(why) | Error::ExternalDependencyFailed(why) => {
                Response::build()
                    .header(ContentType::Plain)
                    .status(Status::InternalServerError)
                    .sized_body(Cursor::new(format!("{}", why)))
                    .ok()
            }
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    BadCount,
    Missing,
    Invaild,
}

pub type Result<T = ()> = std::result::Result<T, Error>;

impl<'a, 'r> FromRequest<'a, 'r> for models::User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("authorization").collect();
        let conn = request.guard::<MainDatabase>()?;
        match keys.len() {
            0 => {
                let mut cookies = request.cookies();
                let tok = cookies.get_private("token");
                match tok {
                    None => Outcome::Failure((Status::Unauthorized, ())),
                    Some(cook) => {
                        let tok = cook.value().to_string();

                        match jwt::verify(tok, conn) {
                            Err(why) => {
                                tracing::error!("JWT verification error: {}", why);
                                Outcome::Failure((Status::Unauthorized, ()))
                            }
                            Ok(user) => Outcome::Success(user),
                        }
                    }
                }
            }
            1 => {
                let tok = keys[0].to_string();
                match jwt::verify(tok, conn) {
                    Err(why) => {
                        tracing::error!("JWT verification error: {}", why);
                        Outcome::Failure((Status::Unauthorized, ()))
                    }
                    Ok(user) => Outcome::Success(user),
                }
            }
            _ => Outcome::Failure((Status::BadRequest, ())),
        }
    }
}
