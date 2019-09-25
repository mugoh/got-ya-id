//! Handles Mail realed functions
use lettre_email::Email;

use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, SmtpTransport, Transport};
use std::{env, io};

/// Enables sending of email
///
/// # Fields
/// - email: Email
///     Creates the email instance holding the data to
///     be sent
/// - sender: SmtpTransport || SmtpClient
///     SMTP Protocol instance to send the email
///     over network
pub struct Mail {
    email: Email,
    mailer: SmtpTransport,
}

/// Returns a Mail struct
impl Mail {
    pub fn new(to_addr: String, to_name: String, subject: String, content: String) -> Mail {
        // ENV
        //let SMTP_CLIENT;
        //let MAIL_USERNAME;
        //let MAIL PASS
        let mut mail_vars: Vec<String> = vec![];
        get_env_var(
            vec!["SMTP_CLIENT", "MAIL_ADDR", "MAIL_USERNAME", "MAIL_PASS"],
            &mut mail_vars,
        );
        let email = Email::builder()
            .to((to_addr, to_name))
            .from(mail_vars[1].clone())
            .subject(subject)
            .alternative(content, "".to_string())
            .build()
            .unwrap();

        let creds = Credentials::new(mail_vars[2].clone(), mail_vars[3].clone());
        let mailer = SmtpClient::new_simple(
            &mail_vars[0], //smtpclient//
        )
        .unwrap()
        .credentials(creds)
        .transport();

        Mail { email, mailer }
    }

    /// Sends the email
    pub fn send(&mut self) -> Result<bool, io::Error> {
        let status = match self.mailer.send(self.email.clone().into()) {
            Ok(_) => Ok(()),
            Err(e) => Result::Err(format!("Couldn't send email , {}", e)),
        };

        Ok(status.is_ok())
    }
}

/// Retrieves ENV Variable given the Keys
pub fn get_env_var<'a>(keys: Vec<&'a str>, values: &mut Vec<String>) -> () {
    for key in keys.iter() {
        let val = env::var(key).unwrap();
        values.push(val);
    }
}

/*
 *
 * fn g() {
    let mut a = Mail::new(
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );
    a.mailer.send(a.email.into());
}
*/
