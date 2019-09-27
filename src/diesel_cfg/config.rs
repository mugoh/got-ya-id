//! Contains the Database setup and configuration
//!

use diesel::{pg::PgConnection, prelude::*};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

pub fn connect_to_db() -> PgConnection {
    dotenv().ok();
    //let user_name = env::var("db_username").expect("Missing database username");
    // let db_pass = env::var("db_pass").expect("Missing database password");
    //let db_addr = env::var("db_addr");
    //let db_name = env::var("db_name").expect("Missing database Name");
    let keys = vec!["db_username", "db_pass", "db_host", "db_port", "db_name"];
    let mut db_vars: HashMap<&str, String> = HashMap::new();

    for key in &keys {
        db_vars.insert(
            key,
            env::var(key).expect(&format!("Missing Dabase config variable: {}", key)),
        );
    }
    let db_url = format!(
        "postgress://{}:{}@{}:{}/{}",
        db_vars.get(&"db_username").unwrap(),
        db_vars.get(&"db_pass").unwrap(),
        db_vars.get(&"db_host").unwrap(),
        db_vars.get(&"db_port").unwrap(),
        db_vars.get(&"db_name").unwrap()
    );
    let db_url = env::var("database_url").expect("Database Url missing");
    PgConnection::establish(&db_url).expect(&format!("Error connecting to {}", &db_url))
}
