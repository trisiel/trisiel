#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use color_eyre::eyre::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rocket::{
    http::{Cookie, Cookies, SameSite},
    response::Redirect,
};
use rocket_contrib::{helmet::SpaceHelmet, json::Json, uuid::Uuid};
use rocket_oauth2::{OAuth2, TokenResponse};

pub mod gitea;
pub mod models;
pub mod schema;

#[database("main_data")]
struct MainDatabase(PgConnection);

struct Gitea;

#[tracing::instrument(skip(conn))]
#[get("/user/<uuid>")]
fn get_user(conn: MainDatabase, uuid: Uuid) -> Json<models::User> {
    use schema::users::dsl::users;
    let result = users
        .find(uuid.into_inner())
        .get_result::<models::User>(&*conn)
        .expect("to find user");

    Json(result)
}

#[tracing::instrument(skip(oauth2, cookies))]
#[get("/login/gitea")]
fn gitea_login(oauth2: OAuth2<Gitea>, mut cookies: Cookies<'_>) -> Redirect {
    oauth2.get_redirect(&mut cookies, &[""]).unwrap()
}

#[tracing::instrument(skip(conn, token, cookies))]
#[get("/auth/gitea")]
fn gitea_callback(
    conn: MainDatabase,
    token: TokenResponse<Gitea>,
    mut cookies: Cookies<'_>,
) -> Redirect {
    let tok = token.access_token().to_string();
    let refresh = token.refresh_token().unwrap().to_string();

    let gitea_user = gitea::user(tok.clone()).exepct("gitea api call to work");

    use schema::users::dsl::{users, email};
    let user: models::User = match users.filter(email.eq(gitea_user.email)).limit(1).load::<models::User>(&conn) {
        Ok(u) => u,
        Err(why) => {
            let u = models::User {
                salutation: gitea_user.full_name,
                email: gitea_user.email,
                is_admin: gitea_user.is_admin,
            }
        }
    };

    // Set a private cookie with the access token
    cookies.add_private(
        Cookie::build("token", token.access_token().to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );

    Redirect::to("/")
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    rocket::ignite()
        .attach(OAuth2::<Gitea>::fairing("gitea"))
        .attach(MainDatabase::fairing())
        .attach(SpaceHelmet::default())
        .mount("/api", routes![get_user])
        .mount("/", routes![gitea_login, gitea_callback])
        .launch();

    Ok(())
}
