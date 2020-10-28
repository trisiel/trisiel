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
use diesel::prelude::*;
use rocket::{
    http::{Cookie, Cookies, SameSite},
    response::Redirect,
};
use rocket_contrib::helmet::SpaceHelmet;
use rocket_oauth2::{OAuth2, TokenResponse};

pub mod api;
pub mod gitea;
pub mod jwt;
pub mod models;
pub mod schema;

#[database("main_data")]
pub struct MainDatabase(PgConnection);

pub struct Gitea;

#[instrument(skip(oauth2, cookies))]
#[get("/login/gitea")]
fn gitea_login(oauth2: OAuth2<Gitea>, mut cookies: Cookies<'_>) -> Redirect {
    oauth2.get_redirect(&mut cookies, &[""]).unwrap()
}

#[instrument(skip(conn, token, cookies))]
#[get("/auth/gitea")]
fn gitea_callback(
    conn: MainDatabase,
    token: TokenResponse<Gitea>,
    mut cookies: Cookies<'_>,
) -> api::Result<String> {
    let tok = token.access_token().to_string();
    let refresh = token.refresh_token().unwrap().to_string();

    let gitea_user =
        gitea::user(tok.clone()).map_err(|why| api::Error::ExternalDependencyFailed(why.into()))?;

    use schema::{
        gitea_tokens, tokens,
        users::{
            dsl::{email, users},
            table as users_table,
        },
    };
    let u: Vec<models::User> = users
        .filter(email.eq(gitea_user.email.clone()))
        .limit(1)
        .load::<models::User>(&*conn)
        .map_err(api::Error::Database)?;

    let user = if u.len() == 0 {
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
            .map_err(api::Error::Database)?;

        let tok = models::NewGiteaToken {
            user_id: u.id.clone(),
            access_token: tok,
            refresh_token: refresh,
        };

        let _: models::GiteaToken = diesel::insert_into(gitea_tokens::table)
            .values(&tok)
            .get_result(&*conn)
            .map_err(api::Error::Database)?;

        info!("new account created for {:?}", u);

        u
    } else {
        info!("{} {:?} logged in", u[0].id, u[0].salutation);
        u[0].clone()
    };

    let tok: models::Token = diesel::insert_into(tokens::table)
        .values(&models::NewToken {
            user_id: user.id.clone(),
        })
        .get_result(&*conn)
        .map_err(api::Error::Database)?;
    info!("created new token for {} with id {}", user.id, tok.id);

    let tok = jwt::make(user.id, tok.id).map_err(api::Error::InternalServerError)?;

    cookies.add_private(
        Cookie::build("token", tok.clone())
            .same_site(SameSite::Lax)
            .finish(),
    );

    Ok(tok)
}

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
                api::whoami,
                api::get_user,
                api::get_tokens,
                api::delete_token,
                api::create_token,
            ],
        )
        .mount("/", routes![gitea_login, gitea_callback])
        .launch();

    Ok(())
}
