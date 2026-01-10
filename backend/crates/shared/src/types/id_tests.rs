use super::*;
use std::str::FromStr;
use uuid::Uuid;

#[test]
fn test_typed_id_creation() {
    let id = UserId::new();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_typed_id_from_uuid() {
    let uuid = Uuid::new_v4();
    let id = UserId::from_uuid(uuid);
    assert_eq!(id.into_inner(), uuid);
}

#[test]
fn test_typed_id_default() {
    let id = UserId::default();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_typed_id_display() {
    let uuid = Uuid::new_v4();
    let id = UserId::from_uuid(uuid);
    assert_eq!(format!("{}", id), uuid.to_string());
}

#[test]
fn test_typed_id_from_str() {
    let uuid = Uuid::new_v4();
    let id = UserId::from_str(&uuid.to_string()).unwrap();
    assert_eq!(id.into_inner(), uuid);
}

#[test]
fn test_typed_id_from_str_error() {
    assert!(UserId::from_str("invalid").is_err());
}
