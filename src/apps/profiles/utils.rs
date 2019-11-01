//! Common functions for the Profiles module

use futures::future::{err, Either};
use futures::{Future, Stream};

use actix_multipart::{Field, MultipartError};
use actix_web::{error as act_err, web, Error};

use serde::{Deserialize, Serialize};

use std::{error, fs::File, io::Write};

use crate::core::py_interface::create_py_mod;

// use crate::core::py_interface::create_py_mod;

/// Extracts the Multipart Field from the multipart object
pub fn extract_multipart_field<'a>(
    field: Field,
) -> impl Future<Item = (i64, String), Error = Error> {
    //
    // dotenv().ok();

    let (file, file_path) = match make_temp_file() {
        Ok(f) => f,
        Err(e) => return Either::A(err(act_err::ErrorInternalServerError(e))),
    };

    //create_py_mod(&file_path);
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                //
                web::block(move || {
                    //
                    //let upload_url =
                    //   env::var("UPLOAD_URL").expect("ENV Err: Missing static image upload URL");

                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("File.write failed: {:?}", e);
                        MultipartError::Payload(act_err::PayloadError::Io(e))
                    })?;
                    acc += bytes.len() as i64;

                    // Sending base64 fails with "url format not recognized"
                    // From the response, The base64 string is taken as a url and not as a serialized bytes
                    // array as would be expected to work.
                    /*
                    System::new("upload")
                        .block_on(lazy(|| {
                            //
                             let data = UploadData {
                                file: bytes.to_vec(),
                                upload_preset: "avatar".to_string(),
                            };
                            println!("{:#?}", data);
                            let client = client::Client::default();
                            client
                                .post(upload_url)
                                .header("User-Agent", "Got ya Id")
                                .send_json(&serde_json::json!(data))
                                .map_err(|e| println!("{:?}", e))
                                .and_then(|response| {
                                    println!("response: {:?}", response);
                                    Ok(())
                                })
                        }))
                        .expect("File uploading failed");
                        */
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
                let path = create_py_mod(file_path).expect("Initiating file send failed");

                (acc, path)
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

/// Holds the fields to include in the file upload body
#[derive(Serialize, Deserialize, Debug)]
struct UploadData {
    #[serde(with = "base_64")]
    file: Vec<u8>,
    upload_preset: String,
}

mod base_64 {
    use base64;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_ = <&str>::deserialize(deserializer)?;
        base64::decode(str_).map_err(de::Error::custom)
    }
}
