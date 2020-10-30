use crate::{jwt, models, MainDatabase};
use color_eyre::eyre::Report;
use rocket::{
    http::{ContentType, Status},
    request::{self, FromRequest, Request},
    response::Responder,
    Outcome, Response,
};
use std::io::Cursor;

pub mod handler;
pub mod token;
pub mod user;

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

    #[error("backblaze error: {0:?}")]
    Backblaze(raze::Error),

    #[error("incorrect number of files uploaded (wanted {0})")]
    IncorrectFilecount(usize),
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
            Error::Backblaze(why) => Response::build()
                .header(ContentType::Plain)
                .status(Status::InternalServerError)
                .sized_body(Cursor::new(format!("b2 error: {:?}", why)))
                .ok(),
            Error::IncorrectFilecount(_) => Response::build()
                .header(ContentType::Plain)
                .status(Status::BadRequest)
                .sized_body(Cursor::new(format!("{}", self)))
                .ok(),
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
                                error!("JWT verification error: {}", why);
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
                        error!("JWT verification error: {}", why);
                        Outcome::Failure((Status::Unauthorized, ()))
                    }
                    Ok(user) => Outcome::Success(user),
                }
            }
            _ => Outcome::Failure((Status::BadRequest, ())),
        }
    }
}
