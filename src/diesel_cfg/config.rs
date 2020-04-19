//! Contains the Database setup and configuration
//!

use diesel::{pg::PgConnection, prelude::*};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

lazy_static! {
    static ref DB_URL: String = {
        dotenv().ok();
        let keys = vec!["db_username", "db_pass", "db_host", "db_port", "db_name"];
        let mut db_vars: HashMap<&str, String> = HashMap::new();

        for key in &keys {
            db_vars.insert(
                key,
                env::var(key).expect(&format!("Missing Database config variable: {}", key)),
            );
        }

        format!(
            "postgres://{}:{}@{}:{}/{}",
            db_vars.get(&"db_username").unwrap(),
            db_vars.get(&"db_pass").unwrap(),
            db_vars.get(&"db_host").unwrap(),
            db_vars.get(&"db_port").unwrap(),
            db_vars.get(&"db_name").unwrap()
        )
    };
}
pub fn connect_to_db() -> PgConnection {
    PgConnection::establish(&DB_URL).expect("Error Initializing the database connection")
}
