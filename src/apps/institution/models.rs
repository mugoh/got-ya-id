use diesel::{self, prelude::*};

use serde::{Deserialize, Serialize};

use validator::Validate;
use validator_derive::Validate;

use regex::Regex;

use chrono::NaiveDateTime;

use std::borrow::Cow;

use crate::{
    apps::{
        ids::models::ClaimableIdentification, ids::validators::validate_str_len,
        profiles::models::Profile, user::models::User, user::utils::from_timestamp,
    },
    diesel_cfg::{config::connect_to_db, schema::institutions},
    errors::error::ResError,
    similarity::cosine::cosine_similarity,
};

/// Insertable institution model
#[derive(Validate, Deserialize, Insertable)]
#[table_name = "institutions"]
#[serde(deny_unknown_fields)]
pub struct NewInstitution<'a> {
    #[validate(length(min = 5, max = 255, message = "Try making the name at least 5 letters"))]
    pub name: Cow<'a, str>,
    #[validate(length(
        min = 3,
        max = 255,
        message = "Try to make the town at least 3 letters long"
    ))]
    pub town: Cow<'a, str>,
    #[validate(length(
        min = 3,
        max = 255,
        message = "Try to make country at least 3 letters long"
    ))]
    pub country: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    postal_address: Option<Cow<'a, str>>,
}

/// Queryable Institution model
#[derive(Queryable, Identifiable, AsChangeset, Serialize, Deserialize)]
#[table_name = "institutions"]
pub struct Institution {
    id: i32,
    pub name: String,
    pub town: String,
    pub country: String,
    description: Option<String>,
    postal_address: Option<String>,
    #[serde(deserialize_with = "from_timestamp")]
    created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    updated_at: NaiveDateTime,
}

/// Institution object for updating
/// changes to the Institution model.
#[derive(Validate, Deserialize, AsChangeset)]
#[table_name = "institutions"]
pub struct UpdatableInstitution<'a> {
    #[validate(custom(function = "validate_str_len"))]
    pub name: Option<Cow<'a, str>>,
    #[validate(custom(function = "validate_str_len"))]
    pub town: Option<Cow<'a, str>>,
    #[validate(custom(function = "validate_str_len"))]
    pub country: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>,
    pub postal_address: Option<Cow<'a, str>>,
}

/// Comparison object for changing
/// a User's institution.
///
/// The name and email fields are used
/// by the function match_institution to
/// verify if the User's email is a close
/// match to the institution they wish to change to.
#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ChangeableInst<'a> {
    /// Name of new Institution
    pub name: Cow<'a, str>,

    /// Institution email
    pub email: Cow<'a, str>,
}

/// Parsable JSON object for changing
/// the User's institution's requests.
#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdatableJsonUserInsitution {
    /// Id of the insitution the User wishes to change to.
    pub institution_id: i32,
    pub user_id: i32,
}

impl<'a> NewInstitution<'a> {
    /// Saves new Institution to the insitutions table.
    pub async fn save(&self) -> Result<Institution, ResError> {
        use crate::diesel_cfg::schema::institutions::dsl::{
            country, institutions as _institutions, name, town,
        };
        let is_present = _institutions
            .filter(
                name.eq(&self.name)
                    .and(town.eq(&self.town))
                    .and(country.eq(&self.country)),
            )
            .load::<Institution>(&connect_to_db())?;

        if is_present.len() > 0 {
            return Err(ResError::new("Institution already exists".into(), 409));
        }

        let created_institution = diesel::insert_into(institutions::table)
            .values(self)
            .get_result::<Institution>(&connect_to_db())?;
        Ok(created_institution)
    }
}

impl Institution {
    /// Retrives all Insitutions the database.
    pub fn get_all() -> Result<Vec<Institution>, ResError> {
        use crate::diesel_cfg::schema::institutions::dsl::institutions as _institutions;
        let all_insitutions = _institutions.load::<Institution>(&connect_to_db())?;
        Ok(all_insitutions)
    }

    /// Finds an Institution by id.
    pub async fn find_by_pk(id: i32) -> Result<Institution, ResError> {
        use crate::diesel_cfg::schema::institutions::dsl::institutions;
        let found_institution = institutions
            .find(id)
            .first::<Institution>(&connect_to_db())?;
        Ok(found_institution)
    }

    pub async fn update(&self, data: &UpdatableInstitution<'_>) -> Result<Institution, ResError> {
        Ok(diesel::update(&*self)
            .set(data)
            .get_result::<Institution>(&connect_to_db())?)
    }

    /// Changes the insitution of a User whose Id
    /// is passed in.
    pub async fn change_user_institution(
        requesting_user: &User,
        updatable_inst: &UpdatableJsonUserInsitution,
    ) -> Result<Institution, ResError> {
        let user_with_profile = User::find_by_pk(updatable_inst.user_id, Some(1))?;
        let user = user_with_profile.0;
        let mut user_profile = user_with_profile.1.unwrap();

        if user.id != requesting_user.id {
            return Err(ResError::new(
                "You are not authorized to change this user's institution".into(),
                401,
            ));
        }

        if let Some(current_user_institution_id) = user_profile.institution_id {
            if current_user_institution_id == updatable_inst.institution_id {
                return Err(ResError::new(
                    "User already belongs to this institution".into(),
                    409,
                ));
            }
        }

        let all_user_emails: Vec<String> = User::all_emails(user.id).await?;

        let new_insitution: Institution = Self::find_by_pk(updatable_inst.institution_id).await?;

        let mut matched_institution = false;
        for email in all_user_emails {
            let changeable_inst: ChangeableInst<'_> = ChangeableInst {
                name: Cow::from(&new_insitution.name),
                email: Cow::from(&email),
            };
            if changeable_inst.is_match().await? {
                matched_institution = true;
                break;
            }
        }

        if !matched_institution {
            return Err(ResError::new(
                format!(
                    "User {} does not have an email matching institution {}",
                    user.username, new_insitution.name
                ),
                400,
            ));
        }

        match ClaimableIdentification::belonging_to_me(&user) {
            Err(_) => Err(ResError::new(
                "User has no identification claim. Create claim first to change institution".into(),
                400,
            )),

            Ok(mut claim) => {
                // Alter db institution refs then:
                // Update user profile, claims, and Ids
                user_profile.institution_id = Some(updatable_inst.institution_id);
                user_profile.save_changes::<Profile>(&connect_to_db())?;

                claim.institution_id = Some(updatable_inst.institution_id);
                claim.save_changes::<ClaimableIdentification>(&connect_to_db())?;
                Ok(new_insitution)
            }
        }
    }
}

impl<'a> ChangeableInst<'a> {
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
