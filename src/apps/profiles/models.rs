//! Models for User Profiles

use crate::apps::user::models::User;
use crate::diesel::RunQueryDsl;
use crate::diesel_cfg::{config::connect_to_db, schema, schema::avatars, schema::profiles};

use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};

use std::{borrow::Cow, error};

/// Holds the User Profile Record
#[derive(
    Queryable, Identifiable, AsChangeset, Associations, Deserialize, Default, Serialize, Debug,
)]
#[belongs_to(User)]
pub struct Profile<'a> {
    id: i32,
    user_id: i32,
    phone: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    institution: Option<String>,
    about: Option<String>,
    found_ids: Option<Cow<'a, i32>>,
}

impl<'a> Profile<'a> {
    /// Finds a given profile by its Primary Key
    pub fn find_by_key(pk: i32) -> Result<Vec<Profile<'a>>, Box<dyn error::Error>> {
        use schema::profiles::dsl::*;
        let query = profiles.find(pk).load(&connect_to_db())?;
        if query.is_empty() {
            return Err(format!("User of ID {} non-existent", pk).into());
        }
        Ok(query)
    }

    /// Retrieves all existing User profiles
    ///
    pub fn retrieve_all<'b>() -> Result<Vec<Profile<'b>>, Box<dyn error::Error>> {
        use crate::diesel_cfg::schema::profiles::dsl::*;
        let prof_vec = profiles.load::<Profile<'b>>(&connect_to_db())?;

        Ok(prof_vec)
    }

    /// Updates a Profile record with the new
    /// data
    pub fn update(&self, new_data: UpdtProfile) -> Result<Profile, diesel::result::Error> {
        //
        //new_data.save_changes::<Profile>(&connect_to_db())

        diesel::update(&*self)
            .set(new_data)
            .get_result::<Profile>(&connect_to_db())
    }
}

/// Holds a new User Profile Record
#[derive(Insertable, Deserialize, Default, Serialize, Debug)]
#[table_name = "profiles"]
#[serde(deny_unknown_fields)]
pub struct NewProfile<'a> {
    user_id: i32,
    phone: Option<Cow<'a, str>>,
    first_name: Option<Cow<'a, str>>,
    middle_name: Option<Cow<'a, str>>,
    last_name: Option<Cow<'a, str>>,
    institution: Option<Cow<'a, str>>,
    about: Option<Cow<'a, str>>,
}

impl<'a> NewProfile<'a> {
    /// Creates a new Profile associated with the user of the given ID.
    ///
    /// # Arguments
    /// - user_id: u32
    ///     ID of the user to associate with this profile
    /// - profile: Option<u32>
    ///     If Some returns the created user Profile object.
    ///     None(default): Nothing is returned
    pub fn new<'b>(user_id: i32, profile: Option<u32>) -> Result<Option<Profile<'b>>, String> {
        let new_profile = NewProfile {
            user_id,
            ..Default::default()
        };
        let res = diesel::insert_into(profiles::table)
            .values(new_profile)
            .get_result::<Profile>(&connect_to_db())
            .expect("Error creating user profile");

        if profile.is_some() {
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}

/// Updatable profile object
#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "profiles"]
pub struct UpdtProfile<'a> {
    phone: Option<Cow<'a, str>>,
    first_name: Option<Cow<'a, str>>,
    middle_name: Option<Cow<'a, str>>,
    last_name: Option<Cow<'a, str>>,
    institution: Option<Cow<'a, str>>,
    about: Option<Cow<'a, str>>,
}

/// User Profile Avatar struct
#[derive(Queryable, Identifiable, AsChangeset, Associations, Deserialize, Serialize, Debug)]
#[belongs_to(User)]
pub struct Avatar<'a> {
    id: i32,
    user_id: i32,
    url: Option<Cow<'a, str>>,
    //file_object: Option<UploadResponse>,
}

/// Insertible profile avatar data
#[derive(Insertable, Deserialize, Serialize, Debug)]
#[table_name = "avatars"]
pub struct NewAvatar<'a> {
    url: Option<Cow<'a, str>>,
    user_id: i32,
}

impl<'a> NewAvatar<'a> {
    /// Creates a new user profile avatar
    pub fn new<'b>(user_id: i32) -> Result<Avatar<'b>, diesel::result::Error> {
        //
        let avatar = NewAvatar {
            url: Some(Cow::Borrowed("default_avatar_url")),
            user_id,
        };

        diesel::insert_into(avatars::table)
            .values(avatar)
            .get_result::<Avatar>(&connect_to_db())
    }
}
