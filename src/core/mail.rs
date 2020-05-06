//! Handles Mail realed functions
use lettre_email::Email;

use lettre::smtp::authentication::Credentials;
use lettre::{smtp, SmtpClient, SmtpTransport, Transport};
use std::{env, process};

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
    pub async fn new<'a>(
        to_addr: &'a str,
        to_name: &'a str,
        subject: &'a str,
        content: &'a str,
    ) -> Result<Mail, String> {
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
            .from(mail_vars[1].as_str())
            .subject(subject)
            .html(content)
            .build();
        let email = match email {
            Ok(em) => em,
            Err(e) => return Err(e.to_string())
        };

        let creds = Credentials::new(
            mail_vars.get(2).unwrap().into(),
            mail_vars.get(3).unwrap().into(),
        );
        let mail_client = SmtpClient::new_simple(
            mail_vars[0].as_ref(), //smtpclient//
        );

        let mailer = if let Err(e) = mail_client {
            return Err(e.to_string());
        } else {
            mail_client.unwrap().credentials(creds).transport()
        };

        Ok(Mail { email, mailer})
    }

    /// Sends the email
    pub async fn send(&mut self) -> Result<smtp::response::Response, smtp::error::Error> {
        self.mailer.send(self.email.clone().into())
    }
}

/// Retrieves ENV Variable given the Keys
pub fn get_env_var<'a>(keys: Vec<&'a str>, values: &mut Vec<String>) {
    for key in keys.iter() {
        let val = env::var((*key).to_lowercase()).unwrap_or_else(|_er| {
            error!("Error configuring mail");
            error!("Missing ENV variable -> {}", key);
            process::exit(0);
        });
        values.push(val);
    }
}
