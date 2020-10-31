use crate::{
    api::{self, Error, Result},
    jwt, models, schema, Gitea, MainDatabase,
};
use color_eyre::eyre::eyre;
use diesel::prelude::*;
use rocket::{
    http::{Cookie, Cookies, SameSite},
    response::Redirect,
};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::{Deserialize, Serialize};

/// A user.
/// https://try.gitea.io/api/swagger#model-User
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub avatar_url: String,
    pub created: String,
    pub email: String,
    pub full_name: String,
    pub id: i64,
    pub is_admin: bool,
    pub language: String,
    pub last_login: String,
    pub login: String,
}

fn user(token: String) -> std::io::Result<User> {
    let resp = ureq::get("https://tulpa.dev/api/v1/user")
        .set("Authorization", &format!("bearer {}", token))
        .set("User-Agent", crate::APP_USER_AGENT)
        .call();
    if !resp.ok() {
        todo!("error here");
    }
    let user: User = resp.into_json_deserialize()?;
    Ok(user)
}

#[instrument(skip(oauth2, cookies))]
#[get("/")]
pub fn login(oauth2: OAuth2<Gitea>, mut cookies: Cookies<'_>) -> Redirect {
    oauth2.get_redirect(&mut cookies, &[""]).unwrap()
}

#[instrument(skip(conn, token, cookies), err)]
#[get("/callback")]
pub fn callback(
    conn: MainDatabase,
    token: TokenResponse<Gitea>,
    mut cookies: Cookies<'_>,
) -> Result<String> {
    let tok = token.access_token().to_string();
    let refresh = token.refresh_token().unwrap().to_string();

    let gitea_user =
        user(tok.clone()).map_err(|why| api::Error::ExternalDependencyFailed(why.into()))?;

    if !gitea_user.is_admin {
        return Err(Error::InternalServerError(eyre!(
            "wasmcloud is not ready for general use yet sorry"
        )));
    }

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
