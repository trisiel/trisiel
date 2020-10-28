use crate::{models, schema, MainDatabase};

use color_eyre::eyre::{eyre, Result};
use diesel::prelude::*;
use hmac::{Hmac, NewMac};
use jwt::{SignWithKey, VerifyWithKey};
use lazy_static::lazy_static;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::env;

lazy_static! {
    pub static ref SECRET: String = env::var("JWT_SECRET")
        .expect("JWT_SECRET to be populated")
        .to_string();
}

#[tracing::instrument]
pub fn make(user_id: uuid::Uuid, token_id: uuid::Uuid) -> Result<String> {
    let key: Hmac<Sha256> = Hmac::new_varkey(&*SECRET.as_bytes()).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", user_id.to_string());
    claims.insert("jti", token_id.to_string());

    let token_str = claims.sign_with_key(&key)?;
    tracing::debug!("token: {}", token_str);
    Ok(token_str)
}

#[tracing::instrument(skip(token, conn))]
pub fn verify(token: String, conn: MainDatabase) -> Result<models::User> {
    use schema::{tokens::dsl::tokens, users::dsl::users};
    let key: Hmac<Sha256> = Hmac::new_varkey(&*SECRET.as_bytes()).unwrap();

    let claims: BTreeMap<String, String> = token.verify_with_key(&key)?;
    let uid = uuid::Uuid::parse_str(
        &claims
            .get("sub")
            .ok_or(eyre!("can't get subscriber from JWT"))?,
    )?;
    let jti = claims
        .get("jti")
        .ok_or(eyre!("can't get token ID from JWT"))?;

    let tok = tokens
        .find(uuid::Uuid::parse_str(&jti)?)
        .get_result::<models::Token>(&*conn)?;

    if tok.deleted_at.is_some() {
        return Err(eyre!("token was deleted at {}", tok.deleted_at.unwrap()));
    }

    if tok.user_id != uid {
        return Err(eyre!("token and user mismatch"));
    }

    let user = users.find(uid).get_result::<models::User>(&*conn)?;

    Ok(user)
}
