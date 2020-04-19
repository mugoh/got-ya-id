//! Contains the Database setup and configuration
//!

use diesel::{pg::PgConnection, prelude::*};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

lazy_static! {
//
}
pub fn connect_to_db() -> PgConnection {
    dotenv().ok();
    let keys = vec!["db_username", "db_pass", "db_host", "db_port", "db_name"];
    let mut db_vars: HashMap<&str, String> = HashMap::new();

    for key in &keys {
        db_vars.insert(
            key,
            env::var(key).expect(&format!("Missing Dabase config variable: {}", key)),
        );
    }
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_vars.get(&"db_username").unwrap(),
        db_vars.get(&"db_pass").unwrap(),
        db_vars.get(&"db_host").unwrap(),
        db_vars.get(&"db_port").unwrap(),
        db_vars.get(&"db_name").unwrap()
    );
    PgConnection::establish(&db_url).expect(&format!("Error connecting to {}", &db_url))
}
