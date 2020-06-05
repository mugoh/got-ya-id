//! Implementations of Http enpoints for the Identifications resource

use actix_web::{web, Error, HttpRequest, HttpResponse, Result};

use super::models::{
    ClaimableIdentification, Identification, MatchedIdtJson, NewClaimableIdt, NewIdentification,
    UpdatableClaimableIdt, UpdatableIdentification,
};
use crate::{
    apps::user::{
        models::User,
        utils::{get_notif_context, TEMPLATE},
    },
    core::{
        mail,
        response::{err, respond},
    },
    errors::error::ResError,
    hashmap,
};

use validator::Validate;

use futures::future::try_join;
use futures::future::TryFutureExt;

use std::env;

/// Receives a json NewIdentification data struct which is
/// used to POST a new Identification
///
/// # url
/// ``
/// # method
/// `POST`
pub async fn create_new_identification(
    mut new_idt: web::Json<NewIdentification<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_idt.0.validate() {
        //return Ok(respond::<serde_json::Value>(hashmap!["status" => "400"], None, Some(&e.to_string())).unwrap());
        return Ok(err("400", e.to_string()));
    }
    let this_user = User::from_token(&req)?;
    new_idt.0.posted_by = Some(this_user.id);

    let idt_f = new_idt.save();

    // Identify possible existing claim on the ID
    let idt_: Identification = Identification::from(&new_idt.0);
    let match_f = idt_.match_claims();

    let (idt, (is_matched, matched_claims)) = try_join(idt_f, match_f).await?;

    // Send notification
    if is_matched {
        send_claim_notification(&idt, matched_claims).await?;
    }

    let res = hashmap!["status" => "201",
            "message" => "Success. Identification created"];

    respond(res, Some(idt), None).unwrap().await
}

///Retrives a single Identification using its PK
///
/// # url
/// `/ids/{id_key}`
///
/// # Method
///  `GET`
pub async fn get_idt(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::find_by_id(*pk)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification retrived"];
    respond(msg, Some(idt), None).unwrap().await
}

/// Retrieves all existing Identifications, found and missing
///
/// ## WARNING
/// Stick to /ids/missing if unsure.
///
/// # Url
/// `/ids/all`
///
/// # Method
/// `GET`
pub async fn get_all_idts() -> Result<HttpResponse, Error> {
    let data = Identification::retrieve_all("all")?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. All identifications retrieved"];

    respond(msg, Some(data), None).unwrap().await
}

/// Retrieves missing Identifications. These are identifications
/// which have not been marked `is_found` as True yet.
///
/// # Url
/// `/ids/missing`
///
/// # Method
/// `GET`
pub async fn get_missing_idts() -> Result<HttpResponse, Error> {
    let data = Identification::retrieve_all("missing")?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Missing identifications retrieved"];

    respond(msg, Some(data), None).unwrap().await
}
/// Retrieves found Identifications. These are identifications
/// which have an `is_found` marked True by the owner.
///
/// # Url
/// `/ids/found`
///
/// # Method
/// `GET`
pub async fn get_found_idts() -> Result<HttpResponse, Error> {
    let data = Identification::retrieve_all("found")?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Found identifications retrieved"];

    respond(msg, Some(data), None).unwrap().await
}
/// Marks an Identification as `found`
///
/// A found IDt is assumed to have been acquired by
/// its owner
///
/// # Url
/// `/ids/found/{key}`
///
/// # METHOD
/// `POST`
///
pub async fn is_now_found(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::mark_found(*pk)?;

    Identification::remove_found_claims(*pk).await?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification status marked FOUND"];

    respond(msg, Some(idt), None).unwrap().await
}

/// Marks an Identifications `is_found` status as
/// False.
///
/// ## INFO
/// This request was initially meant to be made by the
/// owner of the Idt. However, it's not reasonable, as
/// just changing the found status does tell the owner
/// where to find a re-lost Identification.
///
/// The alternative assumption was that posting a new
/// identification should allow the user to search for
/// closely matching fields on existing `found` Idts.
/// In case this happens to be a relost Idt, an update
/// (to (maybe) its new location, and `is_found` status)
/// would simply be done, instead of creating a new Idt
/// item all together.
///
/// This all seems an unnecessay fetch though.
/// Reason? Seems like a fancy way to complicate the
/// work of a user posting a found Idt.
///
/// An Identification whose `is_found` is marked `True`
/// will be considered as good as deleted then, and should
/// in no direct way happen to be visible to the user
///
/// # Url
/// `/ids/lose/{key}`
///
/// # METHOD
/// `POST`
///
pub async fn lose_idt(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::is_lost(pk.into_inner())?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification status marked NOT FOUND"];

    respond(msg, Some(idt), None).unwrap().await
}

/// Updates data in a given Identification
///
/// # Url
/// `/ids/{key}`
///
/// # Method
/// `PUT`
pub async fn update_idt(
    pk: web::Path<i32>,
    new_data: web::Json<UpdatableIdentification<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_data.validate() {
        return err("400", e.to_string()).await;
    };
    let idt = Identification::find_by_id(pk.into_inner())?;
    let saved = idt.update(&req, &new_data)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification updated"];

    let (newly_matched, matched_claims) = idt.match_claims().await?;

    if newly_matched {
        send_claim_notification(&saved, matched_claims).await?;
    }

    // Send Notification

    respond(msg, Some(saved), None).unwrap().await
}

/// Retrieves Identifications belonging to the user
///
/// # Url
/// `/ids/mine`
///
/// # Method
/// GET
///
/// ## Authorization required
pub async fn get_user_idts(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = User::from_token(&req)?;
    let idts = Identification::show_mine(&user)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identifications retrieved"];

    respond(msg, Some(idts), None).unwrap().await
}

/// Retrieves Identifications posted (found) by the user
///
/// # Url
/// `/ids/posted/me`
///
/// # Method
/// GET
///
/// ## Authorization required
pub async fn get_user_posted_idts(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = User::from_token(&req)?;
    let idts = Identification::show_posted_by_me(&user)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identifications retrieved"];

    respond(msg, Some(idts), None).unwrap().await
}

/// Allows a user to claim an Identification as belonging to them
///
/// # Url
/// `/ids/claim/mine`
///
/// # Method
/// `POST`
///
/// # Arguments
/// idt_data: The Identification information to be used in matching
/// the Identification of `idt_key` to the user sending the request
///
/// This data should be an existing Identification Claim
///
/// #### Authentication required
///
/// ## Example
/// ```rust
/// use actix_web::test;
///
/// let data = MatchedIdtJson {idt: 1, claim: 1};
/// let req = test::TestRequest::post()
///     .set_json(&body)
///     .uri(&url)
///     .to_request();
///
/// claim_idt(req);
///
/// ```
pub async fn claim_idt(
    req: HttpRequest,
    data: web::Json<MatchedIdtJson>,
) -> Result<HttpResponse, Error> {
    let owned_idt = Identification::search_matching_claim(&data, &User::from_token(&req)?)?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification claimed"];
    respond(msg, Some(owned_idt), None).unwrap().await
}

/// Created a claim to an identification
/// The claim should have data similar-ish to the Identification
/// the owner of the claim is in search of.
///
/// The Identification the user wants <b>shouldn't neccesarily have been
/// found</b> at the time the claim is being created.
///
/// # Url
/// `/ids/claim`
///
/// # Method
/// `POST`
pub async fn create_idt_claim(
    req: HttpRequest,
    mut new_idt: web::Json<NewClaimableIdt<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_idt.validate() {
        return err("400", e.to_string()).await;
    }

    let new_claim = new_idt.save(&req).await?;
    let match_f = new_claim.match_idt().map_err(|e| e.into());

    let msg = hashmap!["status" => "201",
            "message" => "Success. Claim saved"];
    let resp_f = respond(msg, Some(new_claim.clone()), None).unwrap();

    let (is_matched, res) = try_join(match_f, resp_f).await?;
    // send Notification
    Ok(res)
}

/// Updates existing Claims
///
/// # Url
/// `ids/claim/{key}`
///
/// # Method
/// `PUT`
pub async fn update_idt_claim(
    pk: web::Path<i32>,
    req: HttpRequest,
    idt_data: web::Json<UpdatableClaimableIdt<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = idt_data.validate() {
        return err("400", e.to_string()).await;
    }
    let user = User::from_token(&req)?;
    let claimed_idt = ClaimableIdentification::find_by_id(*pk)?;

    let updated = claimed_idt.update(&user, idt_data.into_inner()).await?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Claimupdated"];

    let newly_matched = claimed_idt.match_idt().await?;

    // Send notification

    respond(msg, Some(updated), None).unwrap().await
}

/// Retrieves Claimable Identifications by PK
///
/// # Url
/// `/ids/claim/{pk}`
///
/// # Method
/// `GET`
pub async fn retrieve_claim(req: HttpRequest, pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    User::from_token(&req)?;

    let idt_claim = ClaimableIdentification::find_by_id(*pk)?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Claim  retrieved"];

    respond(msg, Some(idt_claim), None).unwrap().await
}
/// Retrieves a Claimable Identification belonging to a
/// given user.
///
/// # Url
/// `/ids/claim/user`
///
/// # Method
/// `GET`
pub async fn retrieve_user_claim(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = User::from_token(&req)?;

    let idt_claim = ClaimableIdentification::belonging_to_me(&user)?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Claim  retrieved"];

    respond(msg, Some(idt_claim), None).unwrap().await
}

/// Sends a notification email to Users of the passed Identification
/// claims.
///
/// A notification is sent to every email attached to the User
async fn send_claim_notification(
    idt: &Identification,
    claims: Vec<ClaimableIdentification>,
) -> Result<(), ResError> {
    let claim_rdct_link: String = env::var("CLAIM_REDIRECT_LINK").unwrap_or("".into());

    for claim in claims {
        let user_emails = User::all_emails(claim.user_id)?;
        let user_name = User::find_by_pk(claim.user_id, None)?.0.username;

        let mut context = get_notif_context(&user_name, &claim_rdct_link).await;
        context.insert("id_name", &idt.name);
        context.insert("id_institution", &idt.institution);
        context.insert("id_inst_location", &idt.campus);
        context.insert("id_course", &idt.course);

        let s = TEMPLATE.render("claim_notification.html", &context)?;

        for user_email in user_emails {
            let mut mail = mail::Mail::new(&user_email, &user_name, "Pick up your ID", &s)
                .await
                .map_err(|e| ResError::new(e, 500))?;

            mail.send().await?;
        }
    }
    Ok(())
}
