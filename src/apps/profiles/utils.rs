//! Common functions for the Profiles module

use futures::future::{err, Either};
use futures::{Future, Stream};

use actix_multipart::{Field, MultipartError};
use actix_web::{error as act_err, web, Error};

use std::{error, fs::File, io::Write};

/// Extract the Field from multipart
pub fn extract_multipart_field<'a>(field: Field) -> impl Future<Item = i64, Error = Error> {
    //
    let file = match make_temp_file() {
        Ok(f) => f,
        Err(e) => return Either::A(err(act_err::ErrorInternalServerError(e))),
    };

    println!("Field: {:?}", field);
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                //
                web::block(move || {
                    //
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
            .map(|(_, acc)| acc)
            .map_err(|e| {
                println!("Saving file failed: {:?}", e);
                act_err::ErrorInternalServerError(e)
            }),
    )
}

/// Creates a temprory file to be used in executing the multipart write
fn make_temp_file<'a>() -> Result<File, Box<dyn error::Error>> {
    let rand_str = "temp_upload_file";
    let mut dir = std::env::temp_dir();
    dir.push(rand_str);
    // let f = OpenOptions::new().write(true).create_new(true).open(&dir);
    let f = File::create(dir)?;
    Ok(f)
}
