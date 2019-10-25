//! Common functions for the Profiles module

use futures::Future;

use actix_multipart::Field;
use actix_web::Error;

/// Extract the Field from multipart
pub fn extract_multipart_field<'a>(field: Field) -> impl Future<Item = &'a str, Error = Error> {
    //
    println!("Field: {:?}", field);
    futures::future::ok("Done")
}
