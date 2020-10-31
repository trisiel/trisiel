#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate tracing;

use diesel::pg::PgConnection;

pub mod api;
pub mod b2;
pub mod gitea;
pub mod jwt;
pub mod models;
pub mod schema;

#[database("main_data")]
pub struct MainDatabase(PgConnection);

pub struct Gitea;

// Name your user agent after your app?
pub static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " +https://tulpa.dev/wasmcloud/wasmcloud",
);
