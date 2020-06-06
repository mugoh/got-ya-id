//! Contains the Database setup and configuration

use diesel::{pg::PgConnection, prelude::*};
use dotenv::dotenv;
use std::collections::HashMap;
use std::{borrow::Cow, env};

use crate::apps::user::models::NewUser;

lazy_static! {
    static ref DB_URL: String = {
        dotenv().ok();
        let keys = vec!["db_username", "db_pass", "db_host", "db_port", "db_name"];
        let mut db_vars: HashMap<&str, String> = HashMap::new();

        for key in &keys {
            db_vars.insert(
                key,
                env::var(key)
                    .unwrap_or_else(|_| panic!("Missing Database config variable: {}", key)),
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

/// Seeds a user with admin access level to the database
pub async fn seed_admin_user() {
    let uname = env::var("ADMIN_USERNAME").unwrap_or({
        debug!("Missing env variable: ADMIN_USERNAME. Using default");
        "admin".into()
    });
    let email = env::var("ADMIN_EMAIL").unwrap_or({
        debug!("Missing env variable: ADMIN_EMAIL. Using default");
        "admin@gyid.cow".into()
    });
    let pass = env::var("ADMIN_PASSWORD").unwrap_or({
        debug!("Missing env variable: ADMIN_PASSWORD. Using default");
        "admin".into()
    });

    let mut admin = NewUser {
        username: Cow::Borrowed(&uname),
        password: Cow::Borrowed(&pass),
        access_level: Some(0),
    };
    match admin.save(&email) {
        Ok(_) => debug!("Saved admin user\n"),
        Err(e) => {
            if e.to_string().contains("already") {
                debug!("Admin user present");
            } else {
                error!("Error saving admin user {:?}", e)
            }
        }
    }
}
