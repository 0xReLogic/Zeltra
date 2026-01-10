use super::*;
use crate::config::EmailConfig;

#[tokio::test]
async fn test_new_email_service() {
    let config = EmailConfig::default();
    let service = EmailService::new(config.clone());
    assert_eq!(service.config.smtp_host, config.smtp_host);
}

#[tokio::test]
async fn test_send_verification_email_build() {
    // We cannot easily mock the transport here without significant refactoring
    // or using a mockable transport layer.
    // However, we can test that the method tries to build the email.
    // For unit testing purely, we'd typically check the constructed message,
    // but the `send_email` method consumes the inputs.

    // Ideally, we'd mock the `send_email` or the transport.
    // Since `send_email` is an instance method, we can't easily mock it unless `EmailService` is a trait or uses a trait.

    // For now, we can test the `create_transport` by ensuring it returns an error with invalid config or success with valid.
    // Actually, `create_transport` attempts to build the transport.

    let config = EmailConfig {
        smtp_host: "localhost".to_string(),
        smtp_port: 1025,
        smtp_username: "user".to_string(),
        smtp_password: "password".to_string(),
        from_email: "test@example.com".to_string(),
        from_name: "Test".to_string(),
        frontend_url: "http://localhost:3000".to_string(),
    };

    let service = EmailService::new(config);
    let transport_result = service.create_transport();
    assert!(transport_result.is_ok());
}

#[test]
fn test_email_error_display() {
    assert_eq!(
        format!("{}", EmailError::BuildError("msg".into())),
        "Failed to build email: msg"
    );
    assert_eq!(
        format!("{}", EmailError::SendError("msg".into())),
        "Failed to send email: msg"
    );
    assert_eq!(
        format!("{}", EmailError::InvalidAddress("msg".into())),
        "Invalid email address: msg"
    );
}
