use crate::domain::SubscriberEmail;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{authentication::Credentials, commands::Mail},
    Message, SmtpTransport, Transport,
};
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
    sender: SubscriberEmail,
    smtp_password: Secret<String>,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(sender: SubscriberEmail, smtp_password: Secret<String>, authorization_token: Secret<String>) -> Self {
        Self {
            sender,
            smtp_password,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        // CHECK: Is this a correct usage of `map_err`?
        let sender: Mailbox = format!("Zero2Prod <{}>", self.sender.as_ref())
            .parse()
            .map_err(|_| "Invalid sender email".to_owned())?;
        let recipient: Mailbox = format!(" Ma'facka <{}>", String::from(recipient)).parse()
            .unwrap(); // Unwrap is safe here, since it relies on type safe recipient

        let content = format!("{} — {}", html_content, self.authorization_token.expose_secret());

        let email = Message::builder()
            .from(sender.clone())
            .reply_to(sender)
            .to(recipient)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(content)
            .unwrap();

        let creds = Credentials::new(
            self.sender.clone().into(),
            self.smtp_password.expose_secret().to_owned(),
        );

        // // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {e:?}"),
        }

        Ok(())
    }
}

// NB: Couldn't find a working SMTP mock server for tests.
// Consider running script, which starts a mock server
// #[cfg(test)]
// mod tests {
//     use lettre::{
//         message::Mailboxes,
//         transport::smtp::{
//             authentication::{Credentials, Mechanism},
//             client::{Certificate, Tls, TlsParameters, TlsParametersBuilder},
//             SmtpTransport,
//         },
//         Message, Transport,
//     };
//     use maik::{MailAssertion, MockServer};
//     use std::collections::HashSet;

//     #[test]
// fn no_verify_credentials_no_tls() {
//     // set up and start the mock server
//     let mut server = MockServer::new("smtp.domain.tld");
//     server.set_no_verify_credentials();
//     server.start();

//     // set up the client using lettre
//     let credentials = Credentials::new(
//         String::from("no-reply@domain.tld"),
//         String::from("any_password"),
//     );
//     let mailer = SmtpTransport::relay(&server.host().to_string())
//         .unwrap()
//         .port(server.port())
//         .tls(Tls::None)
//         .credentials(credentials)
//         .authentication(vec![Mechanism::Plain])
//         .build();

//     // send a mail message
//     let message = Message::builder()
//         .from("no-reply@domain.tld".parse().unwrap())
//         .to("user@domain.tld".parse().unwrap())
//         .body(String::from("Here is your verification code: 123456"))
//         .unwrap();
//     println!("1");
//     mailer.send(&message);
//     println!("2");

//     // assert user@domain.tld received some mail that contains "verification code: "
//     let mut recipients = HashSet::new();
//     recipients.insert("user@domain.tld");

//     println!("3");
//     let ma = MailAssertion::new()
//         .recipients_are(recipients)
//         .body_contains("verification code: ");
//     println!("4");
//     assert!(server.assert(ma));
// }
// }
