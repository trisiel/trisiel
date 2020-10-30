use crate::api::Error::Backblaze;
use color_eyre::eyre::{eyre, Result};
use lazy_static::lazy_static;
use raze::{
    api::*,
    util::{self, ReadHashAtEnd, ReadThrottled},
};
use reqwest::blocking::ClientBuilder;
use rocket_upload::Mime;
use std::{env, fs, path::PathBuf};

lazy_static! {
    pub static ref CREDS: String = env::var("B2_CREDFILE")
        .expect("B2_CREDFILE to be populated")
        .to_string();
    pub static ref BUCKET_NAME: String = env::var("B2_MODULE_BUCKET_NAME")
        .expect("B2_MODULE_BUCKET_NAME to be populated")
        .to_string();
}

#[instrument(err)]
pub fn upload(filename: PathBuf, content_type: Mime) -> Result<String> {
    let client = ClientBuilder::new()
        .timeout(None)
        .user_agent(crate::APP_USER_AGENT)
        .build()?;

    let auth = util::authenticate_from_file(&client, CREDS.clone()).map_err(Backblaze)?;
    let upauth = b2_get_upload_url(&client, &auth, "bucket_id").map_err(Backblaze)?;
    let fin = fs::File::open(filename.clone())?;
    let meta = fin.metadata()?;
    let size = meta.len();
    let modf = meta
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        * 1000;
    let ct = content_type.to_string();
    let filepath = filename
        .file_name()
        .ok_or(eyre!("wanted file_name to work"))?
        .to_str()
        .ok_or(eyre!("filename is somehow not utf-8, what"))?;

    let param = FileParameters {
        file_path: filepath.clone(),
        file_size: size,
        content_type: Some(&ct),
        content_sha1: Sha1Variant::HexAtEnd,
        last_modified_millis: modf,
    };

    let reader = fin;
    let reader = ReadHashAtEnd::wrap(reader);
    let reader = ReadThrottled::wrap(reader, 5000);

    let resp = b2_upload_file(&client, &upauth, reader, param).map_err(Backblaze)?;

    Ok(format!("b2://{}/{}", *BUCKET_NAME, filepath))
}
