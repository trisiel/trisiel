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
use rocket_contrib::{helmet::SpaceHelmet};
use rocket_oauth2::{OAuth2, TokenResponse};

pub mod api;
pub mod gitea;
pub mod models;
pub mod schema;

#[database("main_data")]
pub struct MainDatabase(PgConnection);

struct Gitea;

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

    let gitea_user = gitea::user(tok.clone()).expect("gitea api call to work");

    use schema::{
        gitea_tokens,
        users::{
            dsl::{email, users},
            table as users_table,
        },
    };
    let user: models::User = match users
        .filter(email.eq(gitea_user.email.clone()))
        .limit(1)
        .load::<models::User>(&*conn)
    {
        Ok(u) => if u.len() == 0 {
            let u = models::NewUser {
                salutation: gitea_user.full_name,
                email: gitea_user.email,
                is_admin: gitea_user.is_admin,
                is_locked: false,
                tier: 0,
            };

            let u: models::User = diesel::insert_into(users_table)
                .values(&u)
                .get_result(&*conn)
                .expect("able to insert user");

            let tok = models::NewGiteaToken {
                user_id: u.id.clone(),
                access_token: tok,
                refresh_token: refresh,
            };

            let _: models::GiteaToken = diesel::insert_into(gitea_tokens::table)
                .values(&tok)
                .get_result(&*conn)
                .expect("able to insert token");

            u
        } else {
            tracing::info!("{} {:?} logged in", u[0].id, u[0].salutation);
            u[0].clone()
        },
        Err(why) => {
            tracing::error!("error reading from database: {}", why);
            todo!("error response")
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
        .mount("/api", routes![api::get_user])
        .mount("/", routes![gitea_login, gitea_callback])
        .launch();

    Ok(())
}
