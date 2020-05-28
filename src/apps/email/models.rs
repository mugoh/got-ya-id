use diesel::{self, prelude::*};

use validator::Validate;
use validator_derive::Validate;

use serde::{Deserialize, Serialize};

use crate::{
    apps::user::{models::User, utils::from_timestamp},
    diesel_cfg::{config::connect_to_db, schema::emails},
    errors::error::ResError,
};
use chrono::NaiveDateTime;

use std::borrow::Cow;

#[derive(Queryable, Associations, Serialize, Deserialize, AsChangeset, Identifiable)]
#[belongs_to(User)]
//#[table_name = "emails"]
/// Represents the Queryable Email data
pub struct Email {
    id: i32,
    user_id: i32,
    pub email: String,
    /// Selected for default use in identifying the user
    active: bool,
    /// No longer associated. Deleted.
    removed: bool,

    #[serde(deserialize_with = "from_timestamp")]
    pub created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    updated_at: NaiveDateTime,
}

/// Holds new email data
#[derive(Insertable, Deserialize, Validate)]
#[table_name = "emails"]
#[serde(deny_unknown_fields)]
pub struct NewEmail<'a> {
    #[serde(skip_deserializing)]
    pub user_id: i32,

    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'a, str>,
    #[serde(skip_deserializing)]
    pub active: bool,

    #[serde(skip_deserializing)]
    pub removed: bool,
}

impl Email {
    /// Retrieves a User owning a given email
    ///
    /// This is the active email.
    pub fn as_user(curious_email: &str) -> Result<User, diesel::result::Error> {
        use crate::diesel_cfg::schema::emails::dsl::*;
        use crate::diesel_cfg::schema::users::dsl::users;

        let u_id = emails
            .filter(email.eq(&curious_email))
            .select(user_id)
            .get_result::<i32>(&connect_to_db())?;

        let user = users.find(u_id).get_result::<User>(&connect_to_db())?;
        Ok(user)
    }

    /// Retrieves a User owning the given email
    /// returning an empty Vec if the user
    /// doesn't exist.
    ///
    /// This is an alternative to the `as_user` function
    /// which instead returns an Error if the user isn't
    /// found
    pub fn load_user(given_email: &str) -> Result<Vec<User>, diesel::result::Error> {
        use crate::diesel_cfg::schema::emails::dsl::*;
        use crate::diesel_cfg::schema::users::dsl::users;

        let u_id = emails
            .filter(email.eq(&given_email))
            .select(user_id)
            .get_result::<i32>(&connect_to_db())?;

        let user = users.find(u_id).load::<User>(&connect_to_db())?;
        Ok(user)
    }

    /// Returns the User ID identifying the given email.
    pub fn u_id(given_email: &str) -> Result<i32, diesel::result::Error> {
        use crate::diesel_cfg::schema::emails::dsl::*;

        emails
            .filter(email.eq(&given_email))
            .select(user_id)
            .get_result::<i32>(&connect_to_db())
    }

    /// Saves a new email of the given user ID to the database
    pub fn save_email(user: i32, new_email: &str) -> Result<Self, diesel::result::Error> {
        use crate::diesel_cfg::schema::emails::dsl::*;

        diesel::insert_into(emails)
            .values(&(email.eq(new_email), user_id.eq(user)))
            .get_result::<Email>(&connect_to_db())
    }

    /// Marks an Email as removed.
    ///
    /// This is almost equivalent to disassociating
    /// the email with the user's account
    pub fn remove(given_email: &str, usr_id: i32) -> Result<Self, ResError> {
        //
        use crate::diesel_cfg::schema::emails::dsl::{email, emails};

        let mut this_email = emails
            .filter(email.eq(given_email))
            .first::<Self>(&connect_to_db())?;

        if this_email.user_id != usr_id {
            return Err(ResError::unauthorized());
        }

        if this_email.active {
            Err(ResError {
                msg: "Oops, can't remove an active email".into(),
                status: 403,
            })
        } else {
            this_email.removed = true;
            Ok(this_email.save_changes::<Self>(&connect_to_db())?)
        }
    }
}

impl<'a> NewEmail<'a> {
    /// Saves a new email to the Database
    pub fn save(&self) -> Result<Email, ResError> {
        use crate::diesel_cfg::schema::emails::dsl::{email, emails};

        // For previously `removed` emails,
        // undo the remove
        let mut existing_email = emails
            .filter(email.eq(&self.email))
            .load::<Email>(&connect_to_db())?;

        if !existing_email.is_empty() {
            if !existing_email[0].removed {
                return Err(ResError {
                    msg: "Email seems to exist".into(),
                    status: 409,
                });
            } else if existing_email[0].removed && existing_email[0].user_id == self.user_id {
                existing_email[0].removed = false;
                return Ok(existing_email[0].save_changes::<Email>(&connect_to_db())?);
            }
        }

        Ok(diesel::insert_into(emails)
            .values(&*self)
            .get_result::<Email>(&connect_to_db())?)
    }
}
