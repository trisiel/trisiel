#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use color_eyre::eyre::Result;
use rocket_contrib::helmet::SpaceHelmet;
use rocket_oauth2::OAuth2;

use ::wasmcloud::{api, b2, gitea, jwt, Gitea, MainDatabase};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    // XXX(Xe): This looks ineffectual, however it forces jwt::SECRET to be
    // evaluated and will kill the program if JWT_SECRET is not found.
    let _ = *jwt::SECRET;
    let _ = *b2::CREDS;
    let _ = *b2::BUCKET_ID;

    rocket::ignite()
        .attach(OAuth2::<Gitea>::fairing("gitea"))
        .attach(MainDatabase::fairing())
        .attach(SpaceHelmet::default())
        .mount(
            "/api",
            routes![
                api::handler::create,
                api::handler::list,
                api::handler::get,
                api::handler::delete,
                api::handler::get_config,
                api::handler::create_config,
                api::handler::upload_version,
                api::user::whoami,
                api::user::get,
                api::token::list,
                api::token::delete,
                api::token::create,
            ],
        )
        .mount("/login/gitea", routes![gitea::login, gitea::callback])
        .launch();

    Ok(())
}
