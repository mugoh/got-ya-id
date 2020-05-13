//! Identification card models

use super::{utils::serde_pg_point, validators::regexes};
use crate::{
    apps::user::models::{AccessLevel, User},
    apps::user::utils::from_timestamp,
    diesel_cfg::{
        config::connect_to_db,
        schema::{claimed_identifications, identifications},
    },
    errors::error::ResError,
};

use chrono::{NaiveDate, NaiveDateTime};
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use diesel_geometry::data_types::PgPoint;

use std::borrow::Cow;

use actix_web::HttpRequest;

/// Represents the Queryable IDentification data model
/// matching the database `identification` schema
#[derive(Queryable, Associations, Serialize, Deserialize, AsChangeset, Identifiable)]
#[belongs_to(User, foreign_key = "posted_by")]
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
    //    #[serde(with = "naive_date_format")]
    created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    //  #[serde(with = "naive_date_format")]
    updated_at: NaiveDateTime,

    /// Any more relevant info or DESCRIPTION on
    /// the IDt
    about: Option<String>,

    /// The user the Identification belongs to
    owner: Option<i32>,
}

/// The Insertable new Identification record
#[derive(Insertable, Deserialize, Validate)]
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
    about: Option<Cow<'a, str>>,
}

/// Identification model to be used in updating
/// changes to existing identifications
#[derive(AsChangeset, Validate, Deserialize)]
#[table_name = "identifications"]
// #[changeset_for(identifications, behaviour_when_none = "skip")]
// changeset_for unreleased
pub struct UpdatableIdentification<'a> {
    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub name: Option<Cow<'a, str>>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub course: Option<Cow<'a, str>>,

    pub valid_from: Option<NaiveDate>,
    pub valid_till: Option<NaiveDate>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    institution: Option<Cow<'a, str>>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    campus: Option<Cow<'a, str>>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    location_name: Option<Cow<'a, str>>,

    #[serde(flatten, with = "serde_pg_point")]
    location_point: Option<PgPoint>,
    posted_by: Option<i32>,
    about: Option<Cow<'a, str>>,
}

/// The queryable model of claimed identifications
///
/// A Claimable Identification should allow a user to
/// claim ownership of an existing Identification, or
/// to be notified once an Identification matching their
/// particular claim is found.
#[derive(Queryable, Associations, Serialize, Deserialize, AsChangeset, Identifiable)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "claimed_identifications"]
pub struct ClaimableIdentification {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub course: String,
    entry_year: Option<NaiveDate>,
    graduation_year: Option<NaiveDate>,
    institution: String,
    campus_location: String,

    #[serde(deserialize_with = "from_timestamp")]
    created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    updated_at: NaiveDateTime,
}

/// The Insertable model of Claimable Identifications
#[derive(Insertable, Deserialize, Validate)]
#[table_name = "claimed_identifications"]
#[serde(deny_unknown_fields)]
pub struct NewClaimableIdt<'a> {
    #[serde(skip_serializing)]
    pub user_id: i32,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub name: Option<Cow<'a, str>>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub course: Option<Cow<'a, str>>,

    entry_year: Option<NaiveDate>,
    graduation_year: Option<NaiveDate>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    institution: Option<Cow<'a, str>>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    campus_location: Option<Cow<'a, str>>,
}

/// The Insertable model to be used in updating
/// changes to a user-claimed Identifications
#[derive(AsChangeset, Deserialize, Validate)]
#[table_name = "claimed_identifications"]
#[serde(deny_unknown_fields)]
pub struct UpdatableClaimableIdt<'a> {
    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub name: Option<Cow<'a, str>>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub course: Option<Cow<'a, str>>,

    entry_year: Option<NaiveDate>,
    graduation_year: Option<NaiveDate>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    institution: Option<Cow<'a, str>>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    campus_location: Option<Cow<'a, str>>,
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
    pub fn save(&mut self, auth_tk: &HttpRequest) -> Result<Identification, ResError> {
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
                return Err(ResError::new(
                    "You seem to have saved an Identification matching these details".into(),
                    409,
                ));
            }
        }

        let usr_id = User::from_token(auth_tk)?.id;
        self.posted_by = Some(usr_id);

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

    /// Retrieves all existing Identifications
    /// # Returns
    /// An empty vec if none is present
    pub fn retrieve_all() -> Result<Vec<Identification>, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::identifications;

        Ok(identifications.load::<Identification>(&connect_to_db())?)
    }

    /// Marks the identification matching the given key as found
    pub fn mark_found(pk: i32) -> Result<Identification, ResError> {
        let mut idt = Self::find_by_id(pk)?;

        if idt.is_found {
            Err(ResError {
                msg: "Identification found status is True".into(),
                status: 409,
            })
        } else {
            idt.is_found = true;
            idt.save_changes::<Identification>(&connect_to_db())?;
            Ok(idt)
        }
    }

    /// Marks the identification matching the given key as NOT found
    pub fn is_lost(pk: i32) -> Result<Identification, ResError> {
        let mut idt = Self::find_by_id(pk)?;

        if !idt.is_found {
            Err(ResError {
                msg: "Identification found status already False".into(),
                status: 409,
            })
        } else {
            idt.is_found = false;
            idt.save_changes::<Identification>(&connect_to_db())?;
            Ok(idt)
        }
    }

    /// Updates the Idt with the given data
    pub fn update(
        &self,
        auth_tk: &HttpRequest,
        data: &UpdatableIdentification,
    ) -> Result<Identification, ResError> {
        let this_user = User::from_token(auth_tk)?;
        if let Some(pu_id) = self.posted_by {
            if this_user.id != pu_id && this_user.id != AccessLevel::Moderator as i32 {
                return Err(ResError::unauthorized());
            }
        }

        let new_idt = diesel::update(&*self)
            .set(data)
            .get_result::<Identification>(&connect_to_db())?;
        Ok(new_idt)
    }

    /// Retrieves the idenfications that have been posted by the passed user instance.
    ///
    /// These are idts whose `posted_by` matches the user's `id`
    pub fn show_posted_by_me(usr: &User) -> Result<Vec<Identification>, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::{identifications, posted_by};
        let idts = identifications
            .filter(posted_by.eq(usr.id))
            .load::<Self>(&connect_to_db())?;
        Ok(idts)
    }
    /// Retrieves the idenfications that belong to the passed user instance.
    ///
    /// These are idts whose `owner` matches the user's `id`
    pub fn show_mine(usr: &User) -> Result<Vec<Identification>, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::{identifications, owner};
        let idts = identifications
            .filter(owner.eq(usr.id))
            .load::<Self>(&connect_to_db())?;
        Ok(idts)
    }

    /// Mark an Idt's `owner` as the given user
    ///
    /// The User's details should match those of the Identification, to some (probably to be agreed) extent.
    pub fn is_now_mine(&self, usr: &User) -> Result<(), ResError> {
        let is_truly_yours = false;
        Ok(())
    }
}

impl<'a> NewClaimableIdt<'a> {
    /// Saves a new user Identification Claim to db
    pub fn save(&mut self, auth_tk: &HttpRequest) -> Result<ClaimableIdentification, ResError> {
        let this_user = User::from_token(auth_tk)?;
        self.user_id = this_user.id;

        let idt_claim = diesel::insert_into(claimed_identifications::table)
            .values(&*self)
            .get_result::<ClaimableIdentification>(&connect_to_db())?;

        Ok(idt_claim)
    }
}

impl ClaimableIdentification {
    /// Finds an Identification by its primary key
    pub fn find_by_id(key: i32) -> Result<Self, ResError> {
        use crate::diesel_cfg::schema::claimed_identifications::dsl::claimed_identifications;

        let idt = claimed_identifications
            .find(key)
            .get_result::<ClaimableIdentification>(&connect_to_db())?;
        Ok(idt)
    }

    /// Updates the Identification with the passed data
    pub fn update(
        &self,
        auth_tk: &HttpRequest,
        data: UpdatableClaimableIdt,
    ) -> Result<Self, ResError> {
        let this_user = User::from_token(auth_tk)?;

        if this_user.id != self.user_id {
            return Err(ResError::unauthorized());
        }

        let updated_idt = diesel::update(&*self)
            .set(data)
            .get_result::<Self>(&connect_to_db())?;

        Ok(updated_idt)
    }

    /// Get the Claimed Identification that belongs to
    /// this user
    pub fn belonging_to_me(usr: &User) -> Result<Self, ResError> {
        let mut idt_claim =
            ClaimableIdentification::belonging_to(usr).load::<Self>(&connect_to_db())?;
        if idt_claim.is_empty() {
            Err(ResError::not_found())
        } else {
            Ok(idt_claim.pop().unwrap())
        }
    }
}
