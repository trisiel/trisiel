#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use rocket_contrib::{json::Json, uuid::Uuid};

pub mod models;
pub mod schema;

#[database("main_data")]
struct MainDatabase(PgConnection);

#[get("/user/<uuid>")]
fn get_user(conn: MainDatabase, uuid: Uuid) -> Json<models::User> {
    use schema::users::dsl::users;
    let result = users
        .find(uuid.into_inner())
        .get_result::<models::User>(&*conn)
        .expect("to find user");

    Json(result)
}

fn main() {
    rocket::ignite()
        .attach(MainDatabase::fairing())
        .mount("/api", routes![get_user]).launch();
}
