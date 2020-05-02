//! Identification card models

use crate::apps::user::utils::to_year;
use crate::diesel_cfg::{config::connect_to_db, schema::identification};

use chrono::{prelude::*, NaiveDateTime};
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};

/// Represents the Queryable IDentification data model
/// matching the database `identification` schema
#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "identification"]
pub struct Identification {
    pub id: i32,
    pub name: String,
    pub course: String,
    #[serde(deserialize_with = "to_year")]
    pub valid_from: Option<NaiveDateTime>,
    #[serde(deserialize_with = "to_year")]
    pub valid_till: Option<NaiveDateTime>,
    pub institution: String,
    pub campus: Option<String>,
    pub location_name: String,
    pub location_point: Option<(i32, i32)>,
    pub picture: Option<String>,
    posted_by: Option<i32>,
    is_found: bool,
}
