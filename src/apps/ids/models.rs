//! Identification card models

use super::{utils::serde_pg_point, validators::regexes};
use crate::{
    apps::user::utils::from_timestamp,
    diesel_cfg::{config::connect_to_db, schema::identifications},
    errors::error::ResError,
};

use chrono::{NaiveDate, NaiveDateTime};
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

    #[serde(flatten, default, with = "serde_pg_point")]
    /// Lat, Longitude representation of the ID location point
    pub location_point: Option<PgPoint>,

    pub picture: Option<String>,
    posted_by: Option<i32>,
    is_found: bool,

    #[serde(deserialize_with = "from_timestamp")]
    created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    updated_at: NaiveDateTime,
}

/// The Insertable new Identification record
#[derive(Insertable, Debug, Serialize, Deserialize, Validate)]
#[table_name = "identifications"]
#[serde(deny_unknown_fields)]
pub struct NewIdentification<'a> {
    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub name: Cow<'a, str>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub course: Cow<'a, str>,

    pub valid_from: Option<NaiveDate>,
    pub valid_till: Option<NaiveDate>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    institution: Cow<'a, str>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    campus: Cow<'a, str>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    location_name: Cow<'a, str>,

    #[serde(flatten, with = "serde_pg_point")]
    location_point: Option<PgPoint>,
    posted_by: Option<i32>,
}

impl PartialEq<Identification> for NewIdentification<'_> {
    fn eq(&self, idt: &Identification) -> bool {
        let comp_vec = vec![
            self.name.eq(&idt.name),
            self.course.eq(&idt.course),
            self.valid_from.eq(&idt.valid_from),
            self.valid_till.eq(&idt.valid_till),
            self.institution.eq(&idt.institution),
            self.campus.eq(&idt.campus),
            self.location_name.eq(&idt.location_name),
            self.location_point.eq(&idt.location_point),
            self.posted_by.eq(&idt.posted_by),
        ];

        let is_equal = comp_vec.into_iter().all(|v| v);

        // Idts matching in the above details should have been found(logic being an Idt can be
        // relost)
        is_equal & !idt.is_found
    }
}
impl PartialEq<NewIdentification<'_>> for Identification {
    fn eq(&self, idt: &NewIdentification) -> bool {
        let comp_vec = vec![
            self.name.eq(&idt.name),
            self.course.eq(&idt.course),
            self.valid_from.eq(&idt.valid_from),
            self.valid_till.eq(&idt.valid_till),
            self.institution.eq(&idt.institution),
            self.campus.eq(&idt.campus),
            self.location_name.eq(&idt.location_name),
            self.location_point.eq(&idt.location_point),
            self.posted_by.eq(&idt.posted_by),
        ];

        let is_equal = comp_vec.into_iter().all(|v| v);
        is_equal & !self.is_found
    }
}
impl<'a> NewIdentification<'a> {
    /// Saves a new ID record to the Identifications table
    pub fn save(&self) -> Result<Identification, Box<dyn stdErr>> {
        //
        use crate::diesel_cfg::schema::identifications::dsl::{
            campus, course, identifications as _identifications, institution, name,
        };
        let presents = _identifications
            .filter(
                name.eq(&self.name)
                    .and(course.eq(&self.course))
                    .and(institution.eq(&self.institution))
                    .and(campus.eq(&self.campus)),
            )
            .load::<Identification>(&connect_to_db())?;
        for ident in &presents {
            if ident == self {
                return Err(
                    "You seem to have saved an Identification matching these details".into(),
                );
            }
        }

        let idt = diesel::insert_into(identifications::table)
            .values(&*self)
            .get_result::<Identification>(&connect_to_db())?;

        Ok(idt)
    }
}

impl Identification {
    /// Finds an Identification by its primary key
    pub fn find_by_id(key: i32) -> Result<Identification, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::identifications;

        let idt = identifications
            .find(key)
            .get_result::<Identification>(&connect_to_db())?;
        Ok(idt)
    }

    pub fn retrieve_all() -> Result<Vec<Identification>, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::identifications;

        Ok(identifications.load::<Identification>(&connect_to_db())?)
    }
}
