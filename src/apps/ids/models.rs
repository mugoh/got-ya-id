//! Identification card models

use super::utils::serde_pg_point;
use crate::apps::user::utils::to_year;
use crate::diesel_cfg::{config::connect_to_db, schema::identifications};

use chrono::{NaiveDate, NaiveDateTime};
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use diesel_geometry::data_types::PgPoint;
use std::borrow::Cow;
/// Represents the Queryable IDentification data model
/// matching the database `identification` schema
#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "identifications"]
pub struct Identification {
    pub id: i32,
    pub name: String,
    pub course: String,
    #[serde(deserialize_with = "to_year")]
    pub valid_from: NaiveDateTime,
    #[serde(deserialize_with = "to_year")]
    pub valid_till: NaiveDateTime,
    pub institution: String,
    pub campus: Option<String>,
    pub location_name: String,
    pub location_point: Option<(f64, f64)>,
    pub picture: Option<String>,
    posted_by: Option<i32>,
    is_found: bool,
}

/// The Insertable new Identification record
#[derive(Insertable, Serialize, Deserialize, Validate)]
#[table_name = "identifications"]
#[serde(deny_unknown_fields)]
pub struct NewIdentification<'a> {
    pub name: Cow<'a, str>,
    pub course: Cow<'a, str>,
    pub valid_from: NaiveDate,
    pub valid_till: NaiveDate,
    institution: Cow<'a, str>,
    campus: Option<Cow<'a, str>>,
    location_name: Cow<'a, str>,
    #[serde(flatten, with = "serde_pg_point")]
    location_point: PgPoint,
}
