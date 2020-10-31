#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate tracing;

use diesel::prelude::*;
use std::{
    env, fs, io,
    path::PathBuf,
    process::{self, Output},
    time,
};
use uuid::Uuid;
use wasmcloud_api::api::Error::InternalServerError;
use wasmcloud_api::{
    api::{
        Error::{Database, Impossible, Subcommand},
        Result,
    },
    models, schema, MainDatabase,
};

// Name your user agent after your app?
pub static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " +https://tulpa.dev/wasmcloud/wasmcloud",
);

pub static TEMP_FOLDER: &str = concat!(
    "/tmp/",
    env!("CARGO_PKG_NAME"),
    "-",
    env!("CARGO_PKG_VERSION"),
    "/"
);

#[instrument(skip(config), err)]
fn execute(
    handler_id: Uuid,
    config: Vec<models::HandlerConfig>,
    handler_path: PathBuf,
) -> Result<(Output, time::Duration)> {
    let mut child = process::Command::new("/usr/bin/env");
    let child = child.arg("pahi");
    let child = child.arg(handler_path);
    let mut child = child.env("HANDLER_ID", handler_id.to_string());

    for kv in config.into_iter() {
        child = child.env(kv.key_name, kv.value_contents);
    }

    debug!("running");
    let start = time::Instant::now();
    let output = child.output().map_err(Subcommand)?;
    let duration = start.elapsed();

    Ok((output, duration))
}

#[get("/run/<handler_name>")]
#[instrument(skip(conn), err)]
fn schedule(handler_name: String, conn: MainDatabase) -> Result {
    fs::create_dir_all(TEMP_FOLDER)?;
    let hdl = {
        use schema::handlers::dsl::{handlers, human_name};
        handlers
            .filter(human_name.eq(handler_name))
            .first::<models::Handler>(&*conn)
            .map_err(Database)
    }?;

    let cfg = {
        use schema::handler_config::dsl::{handler_config, handler_id};
        handler_config
            .filter(handler_id.eq(hdl.id.clone()))
            .load::<models::HandlerConfig>(&*conn)
            .map_err(Database)
    }?;

    let u = url::Url::parse(&hdl.current_version.ok_or(Impossible)?).map_err(|_| Impossible)?;
    debug!("{:?}", u.host_str().ok_or(Impossible)?);
    // https://cdn.christine.website/file/christine-static/stickers/mara/hacker.png
    let hdl_url = format!(
        "https://cdn.christine.website/file/wasmcloud-modules/{}",
        u.host_str().ok_or(Impossible)?
    );
    let fname = format!("{}{}", TEMP_FOLDER, u.host_str().unwrap());

    debug!(url = &hdl_url[..], fname = &fname[..], "downloading module");
    let resp = ureq::get(&hdl_url).set("User-Agent", APP_USER_AGENT).call();
    if resp.ok() {
        let mut fout = fs::File::create(&fname).map_err(|why| {
            error!("can't make file: {}", why);
            Subcommand(why)
        })?;
        io::copy(&mut resp.into_reader(), &mut fout).map_err(|why| Subcommand(why))?;
    } else {
        error!("while fetching url: {}", resp.status_line());
        return Err(Impossible);
    }

    let (output, duration) = execute(hdl.id, cfg, fname.into()).map_err(|why| {
        error!("error running module: {}", why);
        InternalServerError(why.into())
    })?;
    info!(
        duration = duration.as_millis() as i64,
        module = u.path(),
        "execution finished"
    );

    diesel::insert_into(schema::executions::table)
        .values(&models::NewExecution {
            handler_id: hdl.id,
            finished: true,
            stderr: Some(String::from_utf8(output.stderr).map_err(|_| Impossible)?), // XXX(Cadey): this is not impossible
            execution_time: duration.as_millis() as i32,
        })
        .execute(&*conn)
        .map_err(Database)?;

    Ok(())
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    std::env::set_var("ROCKET_PORT", "8001"); // XXX(Cadey): so I can test both on my machine at once

    rocket::ignite()
        .attach(MainDatabase::fairing())
        .mount("/", routes![schedule])
        .launch();
    Ok(())
}
