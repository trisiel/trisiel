use super::{Error, Result};
use crate::{models, schema, MainDatabase};
use chrono::prelude::*;
use diesel::prelude::*;
use rocket_contrib::{json::Json, uuid::Uuid};
use schema::handlers::dsl::*;
use serde::Deserialize;

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct New {
    pub name: Option<String>,
    pub async_impl: bool,
}

#[instrument(skip(conn), err)]
#[post("/handler", format = "json", data = "<input>")]
pub fn create(
    user: models::User,
    input: Json<New>,
    conn: MainDatabase,
) -> Result<Json<models::Handler>> {
    let input = input.into_inner();
    let name = input.name.unwrap_or(elfs::next().to_lowercase());
    let hdl = diesel::insert_into(schema::handlers::table)
        .values(&models::NewHandler {
            user_id: user.id.clone(),
            human_name: name,
            current_version: None,
            async_impl: input.async_impl,
        })
        .get_result::<models::Handler>(&*conn)
        .map_err(Error::Database)?;

    info!("created handler {} with id {}", hdl.human_name, hdl.id);

    Ok(Json(hdl))
}

#[instrument(skip(conn), err)]
#[get("/handler")]
pub fn list(user: models::User, conn: MainDatabase) -> Result<Json<Vec<models::Handler>>> {
    Ok(Json(
        handlers
            .filter(user_id.eq(user.id))
            .load::<models::Handler>(&*conn)
            .map_err(Error::Database)?,
    ))
}

#[instrument(skip(conn), err)]
#[get("/handler/<uuid>")]
pub fn get(user: models::User, uuid: Uuid, conn: MainDatabase) -> Result<Json<models::Handler>> {
    let uuid = uuid.into_inner();
    let handler = handlers
        .find(uuid)
        .get_result::<models::Handler>(&*conn)
        .map_err(Error::Database)?;

    if handler.user_id != user.id {
        Err(Error::LackPermissions)
    } else {
        Ok(Json(handler))
    }
}

#[instrument(skip(conn), err)]
#[delete("/handler/<uuid>")]
pub fn delete(user: models::User, uuid: Uuid, conn: MainDatabase) -> Result {
    let uuid = uuid.into_inner();

    let hdl: models::Handler = handlers
        .find(uuid.clone())
        .get_result(&*conn)
        .map_err(Error::Database)?;

    if hdl.user_id != user.id && !user.is_admin {
        return Err(Error::LackPermissions);
    }

    diesel::update(handlers.find(uuid))
        .set(deleted_at.eq(Utc::now().naive_utc()))
        .get_result::<models::Handler>(&*conn)?;

    Ok(())
}

#[instrument(skip(conn), err)]
#[get("/handler/<handler_id_str>/config")]
pub fn get_config(
    user: models::User,
    handler_id_str: Uuid,
    conn: MainDatabase,
) -> Result<Json<Vec<models::HandlerConfig>>> {
    let uuid = handler_id_str.into_inner();
    {
        use schema::handler_config::dsl::{handler_config, handler_id};

        let handler = handlers
            .find(uuid)
            .get_result::<models::Handler>(&*conn)
            .map_err(Error::Database)?;

        if handler.user_id != user.id {
            return Err(Error::LackPermissions);
        }

        let config = handler_config
            .filter(handler_id.eq(handler.id))
            .load::<models::HandlerConfig>(&*conn)
            .map_err(Error::Database)?;

        Ok(Json(config))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cfg {
    pub key: String,
    pub value: String,
}

#[instrument(skip(conn, cfg), err)]
#[post("/handler/<handler_id_str>/config", format = "json", data = "<cfg>")]
pub fn create_config(
    user: models::User,
    handler_id_str: Uuid,
    cfg: Json<Vec<Cfg>>,
    conn: MainDatabase,
) -> Result {
    use schema::handler_config::table;
    let uuid = handler_id_str.into_inner();

    let handler = handlers
        .find(uuid)
        .get_result::<models::Handler>(&*conn)
        .map_err(Error::Database)?;

    if handler.user_id != user.id {
        return Err(Error::LackPermissions);
    }

    let cfg: Vec<models::NewHandlerConfig> = cfg
        .into_inner()
        .into_iter()
        .map(|kv| models::NewHandlerConfig {
            key_name: kv.key,
            value_contents: kv.value,
            handler_id: handler.id.clone(),
        })
        .collect();

    diesel::insert_into(table)
        .values(&cfg)
        .get_result::<models::HandlerConfig>(&*conn)
        .map_err(Error::Database)?;

    let _ = cfg.iter().inspect(|kv| info!(name = kv.key_name.as_str(), "config created"));

    Ok(())
}
