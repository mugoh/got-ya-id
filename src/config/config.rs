//! Environment Variables Config

use serde::Deserialize;
use std::error;

/// Holds Env variables deserialized to a struct
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_log")]
    pub rust_log: String,
    pub secret_key: String,
}

/// Default ENV value for log
fn default_log() -> String {
    String::from("actix-web=debug")
}

/// Returns a Serialized ENV variable Configuration
/// struct
pub fn get_env_config() -> Result<Config, Box<dyn error::Error>> {
    match envy::from_env::<Config>() {
        Ok(c) => Ok(c),
        Err(e) => Result::Err(Box::new(e)),
    }
}
