//! Models for User Profiles

use crate::apps::user::models::User;
use crate::diesel::RunQueryDsl;
use crate::diesel_cfg::{config::connect_to_db, schema, schema::avatars, schema::profiles};

use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::{json, value};

use std::{borrow::Cow, error};

/// Holds the User Profile Record
#[derive(Queryable, Identifiable, AsChangeset, Associations, Deserialize, Default, Serialize)]
#[belongs_to(User)]
pub struct Profile<'a> {
    id: i32,
    user_id: i32,
    phone: Option<String>,
    /// Full name
    name: Option<String>,
    institution: Option<String>,
    about: Option<String>,
    found_ids: Option<Cow<'a, i32>>,
}

impl<'a> Profile<'a> {
    /// Finds a given profile by its Primary Key
    ///
    /// # Returns
    ///
    /// (
    ///   [Profile],
    ///   {avatar: url_to_profile avatar}
    ///  )
    pub fn find_by_key(pk: i32) -> Result<(Vec<Profile<'a>>, value::Value), Box<dyn error::Error>> {
        use schema::avatars::dsl::avatars;
        use schema::profiles::dsl::profiles;;

        let profile = profiles.find(pk).load(&connect_to_db())?;
        if profile.is_empty() {
            return Err(format!("User of ID {} non-existent", pk).into());
        }
        let av = json!({"avatar": avatars
        .find(pk)
        .first::<Avatar>(&connect_to_db())?
        .url
        });
        Ok((profile, av))
    }

    /// Retrieves all existing User profiles
    pub fn retrieve_all<'b>() -> Result<Vec<Profile<'b>>, Box<dyn error::Error>> {
        use crate::diesel_cfg::schema::profiles::dsl::profiles;
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
#[derive(Insertable, Deserialize, Default, Serialize)]
#[table_name = "profiles"]
#[serde(deny_unknown_fields)]
pub struct NewProfile<'a> {
    user_id: i32,
    phone: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
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
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'b>(
        user_id: i32,
        profile: Option<u32>,
    ) -> Result<Option<(Profile<'b>, Avatar<'b>)>, String> {
        let new_profile = NewProfile {
            user_id,
            ..Default::default()
        };
        let res = diesel::insert_into(profiles::table)
            .values(new_profile)
            .get_result::<Profile>(&connect_to_db())
            .expect("Error creating user profile");
        let res_av = match NewAvatar::new(user_id) {
            Ok(av) => av,
            Err(e) => return Err(e.to_string()),
        };

        if profile.is_some() {
            Ok(Some((res, res_av)))
        } else {
            Ok(None)
        }
    }
}

/// Updatable profile object
#[derive(AsChangeset, Deserialize)]
#[table_name = "profiles"]
pub struct UpdtProfile<'a> {
    phone: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    institution: Option<Cow<'a, str>>,
    about: Option<Cow<'a, str>>,
}

/// User Profile Avatar struct
#[derive(Queryable, Identifiable, AsChangeset, Associations, Deserialize, Serialize)]
#[belongs_to(User)]
pub struct Avatar<'a> {
    id: i32,
    user_id: i32,
    pub url: Option<Cow<'a, str>>,
    //file_object: Option<UploadResponse>,
}

/// Insertible profile avatar data
#[derive(Insertable, Deserialize)]
#[table_name = "avatars"]
pub struct NewAvatar<'a> {
    url: Option<Cow<'a, str>>,
    user_id: i32,
}

impl<'a> NewAvatar<'a> {
    /// Creates a new user profile avatar
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'b>(user_id: i32) -> Result<Avatar<'b>, diesel::result::Error> {
        //
        //TODO Set Default avatar url
        let default_avatar = "some default avatar url";
        let avatar = NewAvatar {
            url: Some(Cow::Borrowed(default_avatar)),
            user_id,
        };

        diesel::insert_into(avatars::table)
            .values(avatar)
            .get_result::<Avatar>(&connect_to_db())
    }
}
