//! Common functions for the Profiles module

use futures::future::{err, Either};
use futures::{Future, Stream};

use actix_multipart::{Field, MultipartError};
use actix_web::{client, error as act_err, web, Error};

use dotenv::dotenv;
use serde_json::json;

use std::{env, error, fs::File, io::Write};

use crate::core::py_interface::create_py_mod;

// use crate::core::py_interface::create_py_mod;

/// Extract the Field from multipart
pub fn extract_multipart_field<'a>(field: Field) -> impl Future<Item = i64, Error = Error> {
    //
    dotenv().ok();

    let (file, file_path) = match make_temp_file() {
        Ok(f) => f,
        Err(e) => return Either::A(err(act_err::ErrorInternalServerError(e))),
    };

    println!("Field: {:?}", field);
    println!("file path: {}", file_path);

    //create_py_mod(&file_path);
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                //
                web::block(move || {
                    //
                    let client = client::Client::default();
                    let upload_url =
                        env::var("UPLOAD_URL").expect("ENV Err: Missing static image upload URL");
                    client
                        .post(upload_url)
                        .header("User-Agent", "Got ya Id")
                        .send_json(&json!({
                            "file": bytes.as_ref(),
                            "upload_preset": "avatar"
                        }))
                        .map_err(|e| println!("{:?}", e))
                        .and_then(|response| {
                            println!("response: {:?}", response);
                            Ok(())
                        });

                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("File.write failed: {:?}", e);
                        MultipartError::Payload(act_err::PayloadError::Io(e))
                    })?;
                    acc += bytes.len() as i64;
                    Ok((file, acc))
                })
                .map_err(|err: act_err::BlockingError<MultipartError>| {
                    //
                    match err {
                        act_err::BlockingError::Error(err) => err,
                        act_err::BlockingError::Canceled => MultipartError::Incomplete,
                    }
                })
            })
            .map(|(_, acc)| {
                // create_py_mod(file_path).expect("Initiating file sending failed");

                acc
            })
            .map_err(|e| {
                println!("Saving file failed: {:?}", e);
                act_err::ErrorInternalServerError(e)
            }),
    )
}

/// Creates a temprory file to be used in executing the multipart write
fn make_temp_file<'a>() -> Result<(File, String), Box<dyn error::Error>> {
    let rand_str = "temp_upload_file";
    let mut dir = std::env::temp_dir();
    dir.push(rand_str);
    let f_path = dir.to_str().unwrap().to_string();
    let f = File::create(&dir)?;
    Ok((f, f_path))
}
