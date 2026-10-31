use crate::domain::*;
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

#[test]
fn transaction_entry_type_parse_round_trip() {
    assert_eq!(
        "DEBIT".parse::<TransactionEntryType>().unwrap(),
        TransactionEntryType::Debit
    );
    assert_eq!(
        "CREDIT".parse::<TransactionEntryType>().unwrap(),
        TransactionEntryType::Credit
    );
    assert!("FOO".parse::<TransactionEntryType>().is_err());
    assert_eq!(TransactionEntryType::Debit.to_string(), "DEBIT");
}

#[test]
fn account_type_parse_round_trip() {
    for (s, t) in [
        ("ASSET", AccountType::Asset),
        ("LIABILITY", AccountType::Liability),
        ("EQUITY", AccountType::Equity),
        ("INCOME", AccountType::Income),
        ("EXPENSE", AccountType::Expense),
    ] {
        assert_eq!(s.parse::<AccountType>().unwrap(), t);
        assert_eq!(t.to_string(), s);
    }
    assert!("BOGUS".parse::<AccountType>().is_err());
}

#[test]
fn command_state_parse_round_trip() {
    assert_eq!(
        "PENDING".parse::<CommandState>().unwrap(),
        CommandState::Pending
    );
    assert_eq!(
        "APPROVED".parse::<CommandState>().unwrap(),
        CommandState::Approved
    );
    assert_eq!(
        "REJECTED".parse::<CommandState>().unwrap(),
        CommandState::Rejected
    );
    assert_eq!(
        "EXECUTED".parse::<CommandState>().unwrap(),
        CommandState::Executed
    );
    assert_eq!(
        "EXPIRED".parse::<CommandState>().unwrap(),
        CommandState::Expired
    );
    assert!("UNKNOWN".parse::<CommandState>().is_err());
}

#[test]
fn interest_type_parse_round_trip() {
    assert_eq!("FLAT".parse::<InterestType>().unwrap(), InterestType::Flat);
    assert_eq!(
        "REDUCING_BALANCE".parse::<InterestType>().unwrap(),
        InterestType::ReducingBalance
    );
    assert!("WEIRD".parse::<InterestType>().is_err());
}

#[test]
fn parse_enum_error_is_descriptive() {
    let err: ParseEnumError = "NOPE".parse::<AccountType>().unwrap_err();
    assert!(err.to_string().contains("AccountType"));
    assert!(err.to_string().contains("NOPE"));
}
