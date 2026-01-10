use super::*;
use crate::types::money::Currency;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_money_creation() {
    let money = Money::new(Decimal::new(100, 2), Currency::Usd);
    assert_eq!(money.amount, Decimal::new(100, 2));
    assert_eq!(money.currency, Currency::Usd);
}

#[test]
fn test_money_zero() {
    let money = Money::zero(Currency::Usd);
    assert_eq!(money.amount, Decimal::ZERO);
    assert!(money.is_zero());
}

#[test]
fn test_money_negative() {
    let money = Money::new(Decimal::new(-100, 2), Currency::Usd);
    assert!(money.is_negative());
}

#[test]
fn test_currency_display() {
    assert_eq!(format!("{}", Currency::Usd), "USD");
    assert_eq!(format!("{}", Currency::Idr), "IDR");
    assert_eq!(format!("{}", Currency::Eur), "EUR");
    assert_eq!(format!("{}", Currency::Sgd), "SGD");
    assert_eq!(format!("{}", Currency::Jpy), "JPY");
}

#[test]
fn test_currency_from_str() {
    assert_eq!(Currency::from_str("USD").unwrap(), Currency::Usd);
    assert_eq!(Currency::from_str("usd").unwrap(), Currency::Usd);
    assert_eq!(Currency::from_str("IDR").unwrap(), Currency::Idr);
    assert!(Currency::from_str("INVALID").is_err());
}
