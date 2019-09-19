//! This module holds items related to data manipulation
//! for the User Object
//!
use serde::{Deserialize, Serialize};

/// User Object
/// Holds user data
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: u32,
    name: String,
}
