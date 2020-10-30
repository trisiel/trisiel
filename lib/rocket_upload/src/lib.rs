use std::io::{Cursor, Read, Write};
use std::fs::{self, File};
use std::path::Path;
use rocket::data::{self, FromDataSimple};
use rocket::http::Status;
use rocket::{Data, Outcome, Outcome::*, Request};
use multipart::{server::Multipart};

pub use mime::{Mime};

#[derive(Debug)]
pub struct TextPart {
    pub key: String,
    pub value: String,
}
#[derive(Debug)]
pub struct FilePart {
    pub name: String,
    pub path: String,
    pub filename: String,
    pub content_type: Option<Mime>,
}
#[derive(Debug)]
pub struct MultipartDatas {
    pub texts: Vec<TextPart>,
    pub files: Vec<FilePart>,
}

impl FilePart {
    pub fn persist(&self, p: &Path) {
        let s = Path::join(p, &self.filename);
        fs::copy(Path::new(&self.path), &s).unwrap();
    }
}
impl Drop for FilePart {
    fn drop(&mut self) {
        fs::remove_file(Path::new(&self.path)).unwrap();
    }
}
const TMP_PATH: &str = "/tmp/rust_upload/";

impl<'t> FromDataSimple for MultipartDatas {
    type Error = String;

    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, String> {
        let ct = request
            .headers()
            .get_one("Content-Type")
            .expect("no content-type");
        let idx = ct.find("boundary=").expect("no boundary");
        let boundary = &ct[(idx + "boundary=".len())..];

        let mut d = Vec::new();
        data.stream_to(&mut d).expect("Unable to read");

        let mut mp = Multipart::with_body(Cursor::new(d), boundary);
        let mut texts = Vec::new();
        let mut files = Vec::new();

        let mut buffer = [0u8; 4096];

        let mut err_out: Option<Outcome<_, (Status, _), _>> = None;

        mp.foreach_entry(|entry| {
            tracing::debug!("part.headers: {:?}",entry.headers);
            let mut data = entry.data;
            if entry.headers.filename == None {
                let mut text_buffer = Vec::new();

                loop {
                    let c = match data.read(&mut buffer) {
                        Ok(c) => c,
                        Err(err) => {
                            err_out =
                                Some(Failure((Status::UnprocessableEntity, format!("{:?}", err))));
                            return;
                        }
                    };

                    if c == 0 {
                        break;
                    }

                    text_buffer.extend_from_slice(&buffer[..c]);
                }

                let text = match String::from_utf8(text_buffer) {
                    Ok(s) => s,
                    Err(_err) => {
                        err_out = Some(Failure((
                            Status::UnprocessableEntity,
                            ": Data can not read as UTF-8".into(),
                        )));
                        return;
                    }
                };
                tracing::debug!("name: {:?}", entry.headers.name);
                texts.push(TextPart {
                    key: entry.headers.name.to_string(),
                    value: text,
                });
            } else {
                let filename = entry.headers.filename.clone().unwrap();
                if !Path::new(TMP_PATH).exists() {
                    fs::create_dir_all(TMP_PATH).unwrap();
                }

                let target_path = Path::join(Path::new(TMP_PATH), &filename);

                let mut file = match File::create(&target_path) {
                    Ok(f) => f,
                    Err(err) => {
                        err_out =
                            Some(Failure((Status::InternalServerError, format!("{:?}", err))));
                        return;
                    }
                };

                let mut sum_c = 0u64;

                loop {
                    let c = match data.read(&mut buffer) {
                        Ok(c) => c,
                        Err(err) => {
                            try_delete(&target_path);
                            err_out =
                                Some(Failure((Status::UnprocessableEntity, format!("{:?}", err))));
                            return;
                        }
                    };

                    if c == 0 {
                        break;
                    }

                    sum_c = sum_c + c as u64;

                    match file.write(&buffer[..c]) {
                        Ok(_) => (),
                        Err(err) => {
                            try_delete(&target_path);
                            err_out =
                                Some(Failure((Status::InternalServerError, format!("{:?}", err))));
                            return;
                        }
                    }
                }

                tracing::debug!("filename: {:?}", entry.headers.name);
                files.push(FilePart {
                    name: entry.headers.name.to_string(),
                    path: String::from(TMP_PATH) + &filename,
                    filename: entry.headers.filename.clone().unwrap(),
                    content_type: entry.headers.content_type.clone(),
                })
            }
        })
        .unwrap();
        if let Some(failed) = err_out {
            return failed;
        } else {
            let v = MultipartDatas {
                texts: texts,
                files: files,
            };
            return Outcome::Success(v);
        }
    }
}

#[inline]
fn try_delete<P: AsRef<Path>>(path: P) {
    if fs::remove_file(path.as_ref()).is_err() {}
}
