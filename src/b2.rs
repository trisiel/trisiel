use crate::api::Error::Backblaze;
use blake3::Hasher;
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use raze::{
    api::*,
    util::{self, ReadHashAtEnd},
};
use reqwest::blocking::ClientBuilder;
use rocket_upload::Mime;
use std::{
    env, fs,
    io::{self, Read},
    path::PathBuf,
};

lazy_static! {
    pub static ref CREDS: String = env::var("B2_CREDFILE")
        .expect("B2_CREDFILE to be populated")
        .to_string();
    pub static ref BUCKET_ID: String = env::var("B2_MODULE_BUCKET_ID")
        .expect("B2_MODULE_BUCKET_ID to be populated")
        .to_string();
}

fn hash(filename: &PathBuf) -> Result<(String, u64)> {
    let mut fin = fs::File::open(filename)?;
    let mut hasher = Hasher::new();
    let size = copy_wide(&mut fin, &mut hasher)?;
    let hash = hasher.finalize();
    let hash = hash.as_bytes();
    let hash = hex::encode(&hash);

    Ok((hash, size))
}

// A 16 KiB buffer is enough to take advantage of all the SIMD instruction sets
// that we support, but `std::io::copy` currently uses 8 KiB. Most platforms
// can support at least 64 KiB, and there's some performance benefit to using
// bigger reads, so that's what we use here.
fn copy_wide(mut reader: impl Read, hasher: &mut blake3::Hasher) -> io::Result<u64> {
    let mut buffer = [0; 65536];
    let mut total = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(n) => {
                hasher.update(&buffer[..n]);
                total += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

#[instrument(err)]
pub fn upload(filename: PathBuf, content_type: Mime) -> Result<String> {
    let client = ClientBuilder::new()
        .timeout(None)
        .user_agent(crate::APP_USER_AGENT)
        .build()?;

    let auth = util::authenticate_from_file(&client, CREDS.clone()).map_err(Backblaze)?;
    let upauth = b2_get_upload_url(&client, &auth, BUCKET_ID.clone()).map_err(Backblaze)?;
    let fin = fs::File::open(filename.clone())?;
    let meta = fin.metadata()?;
    let (hash, size) = hash(&filename)?;
    let hash = format!("{}.wasm", hash);
    let modf = meta
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        * 1000;
    let ct = content_type.to_string();
    debug!(hash = hash.as_str(), size = size, "uploading to b2");

    let param = FileParameters {
        file_path: hash.as_str(),
        file_size: size,
        content_type: Some(&ct),
        content_sha1: Sha1Variant::HexAtEnd,
        last_modified_millis: modf,
    };

    let reader = fin;
    let reader = ReadHashAtEnd::wrap(reader);

    b2_upload_file(&client, &upauth, reader, param).map_err(Backblaze)?;

    Ok(format!("b2://{}", hash))
}
