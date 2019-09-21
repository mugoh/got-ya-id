//! Environment Variables Config
//!
use envy;
use serde::Deserialize;

/// Holds Env variables deserialized to a struct
#[derive(Deserialize, Debug)]
pub struct Config {
    rust_log: String = "actix-web=info",
}
