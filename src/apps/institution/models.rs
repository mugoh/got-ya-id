use diesel::prelude::*;

use serde::Deserialize;

use validator::Validate;
use validator_derive::Validate;

use regex::Regex;

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
    similarity::cosine::cosine_similarity,
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
    /// Cosine Similarity threshold - .75 (For initials)
    async fn is_match(&self) -> Result<bool, ResError> {
        let match_ = Self::email_institution_sim(&self.name, &self.email, false).await;

        if !match_ {
            Err(ResError {
                msg: "Email doesn't seem to belong to this institution".into(),
                status: 403,
            })
        } else {
            Ok(match_)
        }
    }

    /// Checks if an institution email belongs to the institution
    /// of a given name.
    ///
    /// Two similarity checks are present:
    /// 1. Verifies if the name of the institution is present
    ///    in the email domain.
    /// 2. Checks the similarity of the initials of the institution name
    ///    to the domain substring of the institution email.
    ///
    /// Check No. 2 will only be performed when check 1 fails.
    ///
    /// This bases on the assumption that the domain will be composed
    /// of either (a part of) the name of the institution or the initials.
    ///
    /// ## Arguments:
    /// name: Name of the institution
    ///
    /// email: Institution email
    ///
    /// remove_of: Whether to remove `of` substrings in the institution name
    ///  e.g : `Institution of Weee` -> `Institution Wee`.
    ///  If true, the `of` will not be used for comparison.
    ///
    /// ### Example
    /// ```rust
    /// use got_ya_id::apps::institution::models::ChangeableInst;
    ///
    /// async {
    ///     let name = "Planty Planty";
    ///     let email = "poofy@planty.du";
    ///     assert!(ChangeableInst::email_institution_sim(name, email, false).await);
    /// };
    ///
    /// async {
    ///     let name = "Great Uncookers Dozen";
    ///     let email = "poofy@uncookers.doom";
    ///     assert!(ChangeableInst::email_institution_sim(name, email, false).await);
    ///};
    /// async {
    ///     let name = "Great Uncookers Dozen";
    ///     let email = "poofy@unrs.doom";
    ///     assert_ne!(true, ChangeableInst::email_institution_sim(name, email, false).await);
    /// };
    /// ```
    pub async fn email_institution_sim(name: &str, email: &str, remove_of: bool) -> bool {
        const THRESHOLD: f64 = 0.75;

        let name = name.to_lowercase();
        let email = email.to_lowercase();

        let re = Regex::new(r"\w+").unwrap();

        let inst_name_v = re
            .captures_iter(&name)
            .map(|c| c.get(0).map_or("", |z| z.as_str()))
            .filter(|s| if remove_of { s != &"of" } else { true })
            .collect::<Vec<&str>>();

        let email_dom_v = email.split('@').collect::<Vec<&str>>()[1];

        // remove tail
        let mut email_dom_v = email_dom_v.split('.').collect::<Vec<&str>>();
        email_dom_v.pop();
        let email_dom = email_dom_v.join(".");

        let in_domain = inst_name_v.iter().any(|name| email_dom.contains(name));

        if !in_domain {
            let mut similarity: f64 = 0.0;

            let initials: String = inst_name_v
                .iter()
                .flat_map(|name| name.chars().next())
                .collect();

            if email_dom_v.len() > 1 {
                for sub_dm in email_dom_v {
                    let f_sim = cosine_similarity(&initials, sub_dm).await;

                    if similarity < f_sim {
                        similarity = f_sim;
                    }
                }
            } else {
                similarity = cosine_similarity(&initials, &email_dom).await;
            }

            THRESHOLD < similarity
        } else {
            in_domain
        }
    }
}
