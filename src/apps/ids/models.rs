//! Identification card models

use super::utils::serde_pg_point;
use crate::diesel_cfg::{config::connect_to_db, schema::identifications};

use chrono::NaiveDate;
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use diesel_geometry::data_types::PgPoint;
use std::{borrow::Cow, error::Error as stdErr};
/// Represents the Queryable IDentification data model
/// matching the database `identification` schema
#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "identifications"]
pub struct Identification {
    pub id: i32,
    /// Full name on Identification
    pub name: String,

    /// Major undertaken by holder or the Department
    pub course: String,

    /// Holder's starting Date(y-m-d)
    pub valid_from: Option<NaiveDate>,

    /// Validity end year
    /// Date (y-m-d)
    pub valid_till: Option<NaiveDate>,

    /// The name of the institution the Identification belongs to.
    /// It ought to be its title only  without inclusion of its location
    pub institution: String,

    /// Location/Subtitle defining the exact location
    /// of the institution
    /// e.g `Main, B`
    pub campus: String,

    /// Location from which the ID should be picked
    pub location_name: String,

    #[serde(flatten, with = "serde_pg_point")]
    /// Lat, Longitude representation of the ID location point
    pub location_point: Option<PgPoint>,

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
    pub valid_from: Option<NaiveDate>,
    pub valid_till: Option<NaiveDate>,
    institution: Cow<'a, str>,
    campus: Cow<'a, str>,
    location_name: Cow<'a, str>,
    #[serde(flatten, with = "serde_pg_point")]
    location_point: Option<PgPoint>,
}

impl<'a> NewIdentification<'a> {
    /// Saves a new ID record to the Identifications table
    pub fn save(&mut self) -> Result<Identification, Box<dyn stdErr>> {
        //
        let ID = diesel::insert_into(identifications::table)
            .values(&*self)
            .get_result::<Identification>(&connect_to_db())?;
        Ok(ID)
    }
}
