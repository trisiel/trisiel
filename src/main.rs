#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate tracing;

use color_eyre::eyre::Result;
use diesel::pg::PgConnection;
use rocket_contrib::helmet::SpaceHelmet;
use rocket_oauth2::{OAuth2};

pub mod api;
pub mod gitea;
pub mod jwt;
pub mod models;
pub mod schema;

#[database("main_data")]
pub struct MainDatabase(PgConnection);

pub struct Gitea;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    // XXX(Xe): This looks ineffectual, however it forces jwt::SECRET to be
    // evaluated and will kill the program if JWT_SECRET is not found.
    let _ = *jwt::SECRET;

    rocket::ignite()
        .attach(OAuth2::<Gitea>::fairing("gitea"))
        .attach(MainDatabase::fairing())
        .attach(SpaceHelmet::default())
        .mount(
            "/api",
            routes![
                api::handler::list,
                api::handler::get,
                api::handler::delete,
                api::user::whoami,
                api::user::get,
                api::token::list,
                api::token::delete,
                api::token::create,
            ],
        )
        .mount("/", routes![gitea::login, gitea::callback])
        .launch();

    Ok(())
}
