use super::{Error, Result};
use crate::{b2, models, schema, MainDatabase};
use chrono::prelude::*;
use diesel::prelude::*;
use rocket_contrib::{json::Json, uuid::Uuid};
use rocket_upload::MultipartDatas;
use schema::handlers::dsl::*;
use serde::Deserialize;

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct New {
    pub name: Option<String>,
    pub async_impl: bool,
}

#[post("/handler", format = "json", data = "<input>")]
#[instrument(skip(conn), err)]
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

    info!(
        handler.id = &hdl.id.to_string()[..],
        handler.name = &hdl.human_name[..],
        "created handler"
    );

    Ok(Json(hdl))
}

#[get("/handler")]
#[instrument(skip(conn), err)]
pub fn list(user: models::User, conn: MainDatabase) -> Result<Json<Vec<models::Handler>>> {
    Ok(Json(
        handlers
            .filter(user_id.eq(user.id))
            .load::<models::Handler>(&*conn)
            .map_err(Error::Database)?,
    ))
}

#[get("/handler/<hdl_id>")]
#[instrument(skip(conn), err)]
pub fn get(user: models::User, hdl_id: Uuid, conn: MainDatabase) -> Result<Json<models::Handler>> {
    let uuid = hdl_id.into_inner();
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

#[delete("/handler/<hdl_id>")]
#[instrument(skip(conn), err)]
pub fn delete(user: models::User, hdl_id: Uuid, conn: MainDatabase) -> Result {
    let uuid = hdl_id.into_inner();

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

#[get("/handler/<handler_id_str>/config")]
#[instrument(skip(conn), err)]
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

#[post("/handler/<hdl_id>/config", format = "json", data = "<cfg>")]
#[instrument(skip(conn, cfg), err)]
pub fn create_config(
    user: models::User,
    hdl_id: Uuid,
    cfg: Json<Vec<Cfg>>,
    conn: MainDatabase,
) -> Result {
    use schema::handler_config::table;
    let uuid = hdl_id.into_inner();

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

    let _ = cfg
        .iter()
        .inspect(|kv| info!(name = kv.key_name.as_str(), "config created"));

    Ok(())
}

#[post("/handler/<hdl_id>/upload", data = "<data>")]
#[instrument(skip(conn, data), err)]
pub fn upload_version(
    user: models::User,
    hdl_id: Uuid,
    data: MultipartDatas,
    conn: MainDatabase,
) -> Result<Json<models::Handler>> {
    let uuid = hdl_id.into_inner();

    let handler = handlers
        .find(uuid)
        .get_result::<models::Handler>(&*conn)
        .map_err(Error::Database)?;

    if handler.user_id != user.id {
        return Err(Error::LackPermissions);
    }

    if data.files.len() != 1 {
        return Err(Error::IncorrectFilecount(1));
    }

    let file = data.files.get(0).ok_or(Error::IncorrectFilecount(1))?;
    let ct = file
        .content_type
        .clone()
        .ok_or(Error::IncorrectFilecount(1))?;
    let upload_url = b2::upload(file.path.clone().into(), ct)?;

    let handler = diesel::update(handlers.filter(id.eq(handler.id)))
        .set(current_version.eq(Some(upload_url.clone())))
        .get_result(&*conn)
        .map_err(Error::Database)?;

    info!(url = upload_url.as_str(), "uploaded new version of handler");

    Ok(Json(handler))
}
