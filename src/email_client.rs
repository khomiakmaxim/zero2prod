use crate::domain::SubscriberEmail;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
    sender: SubscriberEmail,
    smtp_password: Option<Secret<String>>,
}

impl EmailClient {
    pub fn new(sender: SubscriberEmail, smtp_password: Option<Secret<String>>) -> Self {
        Self {
            sender,
            smtp_password,
        }
    }

    pub async fn send_email() -> Result<(), String> {
        todo!()
        // let email = Message::builder()
        //     .from("NoBody <nobody@domain.tld>".parse().unwrap())
        //     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        //     .to("Hei <khomiakmaxim@gmail.com>".parse().unwrap())
        //     .subject("Happy new year")
        //     .header(ContentType::TEXT_PLAIN)
        //     .body(String::from("Be happy!"))
        //     .unwrap();

        // let creds = Credentials::new(
        //     "khomiakmaxim@gmail.com".to_owned(),
        //     smtp_password.expose_secret().to_owned(),
        // );

        // // Open a remote connection to gmail
        // let mailer = SmtpTransport::relay("smtp.gmail.com")
        //     .unwrap()
        //     .credentials(creds)
        //     .build();

        // // Send the email
        // match mailer.send(&email) {
        //     Ok(_) => println!("Email sent successfully!"),
        //     Err(e) => panic!("Could not send email: {e:?}"),
        // }

        // Ok(())
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
