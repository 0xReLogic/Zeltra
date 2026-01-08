//! Email service for sending transactional emails.
//!
//! Uses `lettre` for SMTP transport.

use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use thiserror::Error;

use crate::config::EmailConfig;

/// Email service errors.
#[derive(Debug, Error)]
pub enum EmailError {
    /// Failed to build email message.
    #[error("Failed to build email: {0}")]
    BuildError(String),
    /// Failed to send email.
    #[error("Failed to send email: {0}")]
    SendError(String),
    /// Invalid email address.
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),
}

/// Email service for sending transactional emails.
#[derive(Clone)]
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    /// Creates a new email service.
    #[must_use]
    pub const fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Creates an SMTP transport.
    fn create_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, EmailError> {
        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
            .map_err(|e| EmailError::SendError(e.to_string()))?
            .port(self.config.smtp_port)
            .credentials(creds)
            .build()
            .pipe(Ok)
    }

    /// Sends an email verification email.
    ///
    /// # Errors
    ///
    /// Returns an error if the email cannot be sent.
    pub async fn send_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        token: &str,
    ) -> Result<(), EmailError> {
        let verification_url = format!("{}/verify-email?token={}", self.config.frontend_url, token);

        let subject = "Verify your email address - Zeltra";
        let body = format!(
            r"Hi {to_name},

Welcome to Zeltra! Please verify your email address by clicking the link below:

{verification_url}

This link will expire in 24 hours.

If you didn't create an account with Zeltra, you can safely ignore this email.

Best regards,
The Zeltra Team"
        );

        self.send_email(to_email, subject, &body).await
    }

    /// Sends a generic email.
    ///
    /// # Errors
    ///
    /// Returns an error if the email cannot be sent.
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EmailError> {
        let from = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = Message::builder()
            .from(
                from.parse()
                    .map_err(|e| EmailError::InvalidAddress(format!("{e}")))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| EmailError::InvalidAddress(format!("{e}")))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| EmailError::BuildError(e.to_string()))?;

        let transport = self.create_transport()?;
        transport
            .send(email)
            .await
            .map_err(|e| EmailError::SendError(e.to_string()))?;

        Ok(())
    }
}

/// Pipe trait for fluent API.
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert_eq!(config.smtp_host, "localhost");
        assert_eq!(config.smtp_port, 1025);
    }
}
