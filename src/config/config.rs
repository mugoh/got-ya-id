//! Environment Variables Config
//!

use envy;
use lazy_static;
use serde::{self, Deserialize};

lazy_static! {
    static ref ENV_VARS: Result<Config, String> = build_config();
}

/// #[Lazy]
/// Serializes ENV variables to a Config struct
pub fn build_config() -> Result<Config, String> {
    match envy::from_env::<Config>() {
        Ok(c) => Ok(c),
        Err(e) => Result::Err(e.to_string()),
    }
}
/// Holds Env variables deserialized to a struct
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_log")]
    pub rust_log: String,
    pub secret_key: String,
}

/// Default ENV value for log
fn default_log() -> String {
    String::from("actix-web=info")
}

/// Returns a Serialized ENV variable Configuration
/// struct
pub fn get_env_config() -> ENV_VARS {
    ENV_VARS
}
