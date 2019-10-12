//! Models for User Profiles

use crate::diesel_cfg::{config::connect_to_db, schema::profiles};

use serde::{Deserialize, Serialize};

use diesel::{self};

/// Holds the User Profile Record
#[derive(Queryable, Deserialize, Default, Serialize, Debug)]
pub struct Profile {
    id: i32,
    user_id: i32,
    institution: Option<String>,
    phone: Option<String>,
    avatar: Option<String>,
    found_ids: Option<i32>,
}

impl Profile {
    /// Finds a given profile by its Primary Key
    pub fn find_by_key(pk: u32) -> () {
        //
    }
}

/// Holds a new User Profile Record
#[derive(Insertable, Deserialize, Default, Serialize, Debug)]
pub struct NewProfile {
    user_id: i32,
    institution: Option<String>,
    phone: Option<String>,
    avatar: Option<String>,
    found_ids: Option<i32>,
}
//pub struct NewProfile {}
impl NewProfile {
    /// Creates a new Profile associated with the user of the given ID.
    ///
    /// # Arguments
    /// - user_id: u32
    ///     ID of the user to associate with this profile
    /// - profile: Option<u32>
    ///     If Some returns the created user Profile object.
    ///     None(default): Nothing is returned
    pub fn new(user_id: i32, profile: Option<u32>) -> Result<Option<NewProfile>, String> {
        let new_profile = NewProfile {
            user_id,
            ..Default::default()
        };
        let res = diesel::insert_into(profiles::table)
            .values(&new_profile)
            .get_result(&connect_to_db())
            .expect("Error creating user profile");
        match res {
            Ok(p) => {
                if profile.is_some() {
                    Ok(Some(p))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err("Unable to create user profile".to_string()),
        }
    }
}
