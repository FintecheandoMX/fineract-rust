use crate::ids::*;
use crate::money::{Currency, Money};
use crate::tenant::TenantContext;

#[test]
fn core_types_compile_together() {
    let tenant = TenantContext::new(TenantId::new(1));
    let _ = tenant;
    let money = Money::new(rust_decimal::Decimal::new(100, 2), Currency::USD);
    assert!(money.is_positive());
    let _ = money;
}

#[test]
fn currency_validation_rejects_invalid() {
    assert!(Currency::from_code("usd").is_err());
    assert!(Currency::from_code("US").is_err());
    assert!(Currency::from_code("USD").is_ok());
    assert_eq!(Currency::USD.code(), "USD");
}

#[test]
fn money_zero_is_not_positive() {
    let zero = Money::zero(Currency::MXN);
    assert!(!zero.is_positive());
    assert!(zero.is_non_positive());
}
