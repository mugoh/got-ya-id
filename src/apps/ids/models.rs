//! Identification card models

use super::{utils::serde_pg_point, validators::regexes};
use crate::{
    apps::user::models::{AccessLevel, User},
    apps::user::utils::from_timestamp,
    diesel_cfg::{
        config::connect_to_db,
        schema::{claimed_identifications, identifications, matched_identifications},
    },
    errors::error::ResError,
    similarity::cosine::cosine_similarity,
};

use chrono::{NaiveDate, NaiveDateTime};
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use diesel_geometry::data_types::PgPoint;

use std::borrow::Cow;

use actix_web::HttpRequest;

/// Represents a matched Identification-Claim
#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "matched_identifications"]
struct MatchedIDt {
    pub id: i64,
    claim_id: i32,
    identification_id: i32,

    #[serde(deserialize_with = "from_timestamp")]
    created_at: NaiveDateTime,
}
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
    #[serde(skip_deserializing)]
    pub user_id: i32,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub name: Cow<'a, str>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub course: Option<Cow<'a, str>>,

    entry_year: Option<NaiveDate>,
    graduation_year: Option<NaiveDate>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    institution: Cow<'a, str>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    campus_location: Option<Cow<'a, str>>,
}

/// The Insertable model to be used in updating
/// changes to a user-claimed Identifications
#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "claimed_identifications"]
#[serde(deny_unknown_fields)]
pub struct UpdatableClaimableIdt<'a> {
    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub name: Option<Cow<'a, str>>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub course: Option<Cow<'a, str>>,

    pub entry_year: Option<NaiveDate>,
    pub graduation_year: Option<NaiveDate>,

    #[validate(regex(path = "regexes::ALPHA_REGEX", message = "should just have letters"))]
    pub institution: Option<Cow<'a, str>>,

    #[validate(regex(
        path = "regexes::LOCATION_REGEX",
        message = "should have letters, digits or -_`"
    ))]
    pub campus_location: Option<Cow<'a, str>>,
}

/// Json Model for an Identification claim request
#[derive(Deserialize)]
pub struct MatchedIdtJson {
    /// The identification ID a User wants to claim
    idt: i32,
    /// A (possibly) matching Claimable Identification ID
    claim: i32,
}

impl PartialEq<NewClaimableIdt<'_>> for ClaimableIdentification {
    fn eq(&self, claim: &NewClaimableIdt) -> bool {
        let match_fields = [
            self.entry_year.eq(&claim.entry_year),
            self.graduation_year.eq(&claim.graduation_year),
            if claim.course.is_some() {
                self.course.eq(claim.course.as_ref().unwrap())
            } else {
                false
            },
            if let Some(ref loc) = claim.campus_location {
                self.campus_location.eq(loc)
            } else {
                false
            },
            self.institution.eq(&claim.institution),
            self.name.eq(&claim.name),
        ];

        match_fields.iter().all(|&field| field)
    }
}

impl PartialEq<ClaimableIdentification> for NewClaimableIdt<'_> {
    fn eq(&self, claim: &ClaimableIdentification) -> bool {
        let match_fields = [
            self.entry_year.eq(&claim.entry_year),
            self.institution.eq(&claim.institution),
            self.name.eq(&claim.name),
            self.graduation_year.eq(&claim.graduation_year),
            if self.course.is_some() {
                self.course.as_ref().unwrap().eq(&claim.course)
            } else {
                false
            },
            if let Some(loc) = &self.campus_location {
                claim.campus_location.eq(loc)
            } else {
                false
            },
        ];

        match_fields.iter().all(|&field| field)
    }
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
    pub async fn save(&mut self, auth_tk: &HttpRequest) -> Result<Identification, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::{
            campus, course, identifications as _identifications, institution, name,
        };
        let usr_id = User::from_token(auth_tk)?.id;
        self.posted_by = Some(usr_id);

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

        let idt = diesel::insert_into(identifications::table)
            .values(&*self)
            .get_result::<Identification>(&connect_to_db())?;

        Ok(idt)
    }

    /// Finds Identification claims that would be possible matches
    /// to a new Identification.
    ///
    /// This method should be analogous to `NewClaim.match_idt`
    pub async fn match_claims(&self) -> Result<(), ResError> {
        use crate::diesel_cfg::schema::claimed_identifications::dsl::{
            campus_location, claimed_identifications, institution,
        };

        let idt_claims = claimed_identifications
            .filter(
                institution
                    .eq(&self.institution)
                    .and(campus_location.eq(&self.campus)),
            )
            .load::<ClaimableIdentification>(&connect_to_db())?;

        for claim in idt_claims.iter() {
            if self.is_possible_match(claim).await {
                MatchedIDt::save(claim, &self.into()).await?;
            }
        }
        Ok(())
    }

    /// Finds the similarity between a Claim and this Identification,
    /// returning true if the Claim is a  possible match.
    ///
    /// Matching metric is cosine similarity.
    pub async fn is_possible_match(&self, claim: &ClaimableIdentification) -> bool {
        ClaimableIdentification::is_matching_idt(claim, &self.into()).await
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
    ///
    /// # Arguments
    /// ## status: &str
    /// found - Retrieve found Idts
    /// missing - Retrieve non-found Idts
    /// all - Retrieve all idts
    ///
    /// The default is missing, for a status argument that
    /// does not match any of the three options.
    ///
    /// # Returns
    /// An empty vec if none is present
    pub fn retrieve_all(status: &str) -> Result<Vec<Identification>, ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::{identifications, is_found as found};

        let idts = if status == "all" {
            identifications.load::<Identification>(&connect_to_db())?
        } else if status == "found" {
            identifications
                .filter(found.eq(true))
                .load::<Self>(&connect_to_db())?
        } else {
            identifications
                .filter(found.eq(false))
                .load::<Self>(&connect_to_db())?
        };

        Ok(idts)
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

    /// Removes Identification claim matches of the Identification id given. `key`.
    ///
    /// This is meant to be called once an Identification has been found, and a claim
    /// is no longer needed.
    pub async fn remove_found_claims(key: i32) -> Result<(), ResError> {
        use crate::diesel_cfg::schema::matched_identifications::dsl::{
            identification_id, matched_identifications,
        };
        diesel::delete(matched_identifications.filter(identification_id.eq(key)))
            .execute(&connect_to_db())?;
        Ok(())
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
    pub fn is_now_mine(&self, _usr: &User) -> Result<(), ResError> {
        let _is_truly_yours = false;
        Ok(())
    }

    /// Checks if the Identification and Claim IDs given in
    /// the MatchedIdtJson request match each other.
    pub fn search_matching_claim(
        data: &MatchedIdtJson,
        usr: &User,
    ) -> Result<Identification, ResError> {
        use crate::diesel_cfg::schema::claimed_identifications::dsl::claimed_identifications;
        use crate::diesel_cfg::schema::identifications::dsl::identifications;
        use crate::diesel_cfg::schema::matched_identifications::dsl::*;

        let idt_match = matched_identifications
            .filter(claim_id.eq(data.claim).and(identification_id.eq(data.idt)))
            .first::<MatchedIDt>(&connect_to_db())?;

        let this_claim = claimed_identifications
            .find(idt_match.claim_id)
            .first::<ClaimableIdentification>(&connect_to_db())?;

        if this_claim.user_id != usr.id {
            return Err(ResError::unauthorized());
        }

        let mut this_idt = identifications
            .find(idt_match.identification_id)
            .first::<Identification>(&connect_to_db())?;

        this_idt.owner = Some(usr.id);
        let saved_idt = this_idt.save_changes::<Identification>(&connect_to_db())?;
        Ok(saved_idt)
    }
}

impl<'a> NewClaimableIdt<'a> {
    /// Saves a new user Identification Claim to db
    pub async fn save(
        &mut self,
        auth_tk: &HttpRequest,
    ) -> Result<ClaimableIdentification, ResError> {
        use crate::diesel_cfg::schema::claimed_identifications::dsl::{
            claimed_identifications as cl_idt_table, institution as c_institution, name as c_name,
        };

        let this_user = User::from_token(auth_tk)?;
        self.user_id = this_user.id;
        self.has_claim(&this_user).await?;

        let existing_claims = cl_idt_table
            .filter(
                c_name
                    .eq(self.name.as_ref())
                    .and(c_institution.eq(self.institution.as_ref())),
            )
            .load::<ClaimableIdentification>(&connect_to_db())?;
        self.is_unique(&existing_claims)?;

        let idt_claim = diesel::insert_into(claimed_identifications::table)
            .values(&*self)
            .get_result::<ClaimableIdentification>(&connect_to_db())?;

        idt_claim.match_idt().await?;

        Ok(idt_claim)
    }

    /// Checks whether a new Claim has fields, which ought to be unique,  matching another
    pub fn is_unique(
        &self,
        existing_claims: &Vec<ClaimableIdentification>,
    ) -> Result<bool, ResError> {
        let duplicate = existing_claims.into_iter().any(|claim| claim == self);
        if duplicate {
            Err(ResError {
                msg: "A similar Identification claim seems to exist".into(),
                status: 409,
            })
        } else {
            Ok(!duplicate)
        }
    }

    /// Checks if a User has an existing claim.
    ///
    /// Users should make just one claim by default,
    /// which they can then give controlled edits.
    pub async fn has_claim(&self, current_user: &User) -> Result<bool, ResError> {
        //
        let user_claims = ClaimableIdentification::belonging_to(current_user)
            .load::<ClaimableIdentification>(&connect_to_db())?;
        match user_claims.is_empty() {
            false => Err(ResError {
                msg: "Dude, you created a claim some while back".into(),
                status: 409,
            }),
            true => Ok(false),
        }
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
    ///
    /// This could also be done if a re-match of the claim
    /// to existing, or newly posted Identifications is neccessary.
    pub async fn update(
        &self,
        this_user: &User,
        data: UpdatableClaimableIdt<'_>,
    ) -> Result<Self, ResError> {
        if this_user.id != self.user_id {
            return Err(ResError::unauthorized());
        }

        let updated_idt = diesel::update(&*self)
            .set(data)
            .get_result::<Self>(&connect_to_db())?;
        self.match_idt().await?;

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

    /// Matches Identifications that would belong to a claim.
    ///
    /// Match criteria is based on similarity between the Identification
    /// details and those given on the Claim, more weight being given
    /// to some fields deemed to be commonly unique, (such as the holder's name)
    ///
    /// The Identifications selected for match are selected from the name of the
    /// institution given in the Claim.
    pub async fn match_idt(&self) -> Result<(), ResError> {
        use crate::diesel_cfg::schema::identifications::dsl::{
            campus, identifications, institution, is_found,
        };

        let idts = if !self.campus_location.is_empty() {
            let campus_loc = &self.campus_location;
            identifications
                .filter(
                    institution
                        .eq(&self.institution)
                        .and(campus.eq(campus_loc))
                        .and(is_found.eq(false)),
                )
                .load::<Identification>(&connect_to_db())?
        } else {
            identifications
                .filter(institution.eq(&self.institution).and(is_found.eq(false)))
                .load::<Identification>(&connect_to_db())?
        };

        self.find_similarity(idts).await?;
        Ok(())
    }

    /// Compares the fields of a claim to given Identifications to ascertain
    /// if the claim could refer to any of them.
    ///
    /// The similarity metric is cosine distance of the Identification and the
    /// Claim fields
    async fn find_similarity(
        &self,
        idents: Vec<Identification>,
    ) -> Result<(), diesel::result::Error> {
        for idt in idents.iter() {
            if Self::is_matching_idt(self, idt).await {
                MatchedIDt::save(self, idt).await?;
            }
        }
        Ok(())
    }

    /// Finds the similarity between a Claim and an Identification,
    /// returning true if they match.
    ///
    /// Cosine threshold: .90
    async fn is_matching_idt(claim: &ClaimableIdentification, idt: &Identification) -> bool {
        // Use cosine similarity here
        // name, course,
        // valid-from, valid-till
        // Assign an overall percentage match for each of the used fields
        // Significance: name: .55, course: .15, valids: .3 each

        let mut overall_significance: f64 = 0.0;

        // Min threshold from matching name only -> .90*.6
        let min_threshold = 0.54;

        let nm_sig: f64 = 0.6; // Cosine of 1 contributes .60
        let crse_sig: f64 = 0.25; // Cosine of 1 contributes .25
        let tm_sig: f64 = 0.15; // Valid from, Valid till each contribute tm_sig/2

        let name_s = cosine_similarity(claim.name.as_ref(), &idt.name).await;
        overall_significance += name_s * nm_sig;

        let course_s = cosine_similarity(claim.course.as_ref(), &idt.course).await;
        overall_significance += course_s * crse_sig;

        if idt.valid_from.is_some() && claim.entry_year.is_some() {
            if claim.entry_year.unwrap().eq(&idt.valid_from.unwrap()) {
                overall_significance += tm_sig / 2.;
            }
        } else if idt.valid_till.is_some() && claim.graduation_year.is_some() {
            if claim.graduation_year.unwrap().eq(&idt.valid_till.unwrap()) {
                overall_significance += tm_sig / 2.;
            }
        }
        overall_significance >= min_threshold
    }
}

impl MatchedIDt {
    /// Inserts a new Identification/Claim match into the Matches
    /// table.
    pub async fn save(
        claim: &ClaimableIdentification,
        idt: &Identification,
    ) -> Result<usize, diesel::result::Error> {
        use crate::diesel_cfg::schema::matched_identifications::dsl::*;
        use diesel::pg::upsert::on_constraint;

        // unique (claim_id, identification_id)
        // Ignore this, it's bound to happen
        Ok(diesel::insert_into(matched_identifications)
            .values(&(claim_id.eq(claim.id), identification_id.eq(idt.id)))
            .on_conflict(on_constraint("matched_claim_id_unique"))
            .do_nothing()
            .execute(&connect_to_db())?)
    }
}

impl std::convert::From<&NewIdentification<'_>> for Identification {
    /// Desired fields are those used in comparison between Identifications
    /// ans ClaimableIdentifications.
    ///
    /// Usable fields are only:
    /// name, course, campus, valid_from, valid_till, institution
    fn from(new_idt: &NewIdentification<'_>) -> Self {
        Identification {
            name: new_idt.name.as_ref().into(),
            course: new_idt.course.as_ref().into(),
            campus: new_idt.campus.as_ref().into(),
            valid_from: new_idt.valid_from,
            valid_till: new_idt.valid_till,
            institution: new_idt.institution.as_ref().into(),

            // Below fields should NOT be used on an Idt converted from a NewIdt
            id: 0,
            created_at: NaiveDate::from_ymd(2010, 01, 01).and_hms(0, 00, 00),
            updated_at: NaiveDate::from_ymd(2010, 01, 01).and_hms(0, 00, 00),
            location_name: "".into(),
            location_point: None,
            picture: None,
            posted_by: new_idt.posted_by,
            is_found: false,
            about: None,
            owner: None,
        }
    }
}
