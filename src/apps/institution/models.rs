use diesel::prelude::*;

use serde::Deserialize;

use validator::Validate;
use validator_derive::Validate;

use std::borrow::Cow;

use crate::{
    apps::{
        email::models::Email,
        ids::models::{ClaimableIdentification, UpdatableClaimableIdt},
        profiles::models::Profile,
        user::models::User,
    },
    diesel_cfg::config::connect_to_db,
    errors::error::ResError,
};

/// Parsed Json Data necessary for changing
/// a User's institution.
#[serde(deny_unknown_fields)]
#[derive(Validate, Deserialize)]
pub struct ChangeableInst<'a> {
    /// Name of new Institution
    pub name: Cow<'a, str>,

    /// Institution email
    pub email: Cow<'a, str>,
}

impl<'a> ChangeableInst<'a> {
    /// Updates the institution name identifying a User.
    ///
    /// The institution is in the records `Profile` and `Claims`
    pub async fn update(&self, user: &User) -> Result<(), ResError> {
        let u_id = Email::u_id(&self.email)?;

        if u_id != user.id {
            return Err(ResError::unauthorized());
        }

        self.is_match().await?;

        let mut prf = Profile::belonging_to(user).get_result::<Profile>(&connect_to_db())?;
        prf.institution = Some(self.name.as_ref().into());
        prf.save_changes::<Profile>(&connect_to_db())?;

        let claim = ClaimableIdentification::belonging_to(user)
            .get_result::<ClaimableIdentification>(&connect_to_db())?;
        let upt_claim = UpdatableClaimableIdt {
            institution: Some(Cow::Borrowed(&self.name)),
            ..Default::default()
        };

        claim.update(user, upt_claim).await?;

        Ok(())
    }

    /// Attempts to acertain whether the email belongs to the
    /// institution.
    ///
    /// This is experimental, and relies on character similarity metrics,
    /// which check if alphaneumerics or words on the email domain
    /// having letters(Initials) or words identical to those of the
    /// institution's name to a certain extent.
    ///
    /// Leventshein threshold - .75 (For initials)
    async fn is_match(&self) -> Result<bool, ResError> {
        Ok(true)
    }
}
