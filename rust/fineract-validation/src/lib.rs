// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. See the License for the
// specific language governing permissions and limitations
// under the License.

//! Validation framework for Fineract domain input.
//!
//! Hand-written, dependency-light validators with clear, field-scoped error
//! messages. Avoids `proc-macro` crates (e.g. `validator`) to keep the supply
//! chain small and auditable.
//!
//! Security: all validation runs at the system boundary. Reject unexpected /
//! invalid input early; never coerce or silently drop invalid values. Multiple
//! violations are collected into a single [`ValidationResult`] so clients
//! receive every problem at once rather than one at a time.

#![forbid(unsafe_code)]

use std::convert::TryFrom;

use fineract_core::error::DomainError;
use fineract_core::money::{Currency, Money};

/// A validation error scoped to a single field.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("field '{field}' must not be empty")]
    Empty { field: String },
    #[error("field '{field}' is too long (max {max} characters, got {got})")]
    TooLong {
        field: String,
        max: usize,
        got: usize,
    },
    #[error("field '{field}' must be a valid currency code")]
    InvalidCurrency { field: String },
    #[error("field '{field}' must be a positive amount")]
    NonPositive { field: String },
    #[error("field '{field}' must not be negative")]
    Negative { field: String },
    #[error("field '{field}' must be a valid identifier")]
    InvalidId { field: String },
    #[error("field '{field}' is missing")]
    Missing { field: String },
    #[error("field '{field}' is not a valid email address")]
    InvalidEmail { field: String },
    #[error("field '{field}' is not a valid phone number")]
    InvalidPhone { field: String },
    #[error("{0}")]
    Custom(String),
}

/// Collected outcome of validating one or more fields.
///
/// Holds every violation found so a boundary can report all problems in a
/// single 400 response instead of failing on the first one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Convert into a `Result`, producing all collected errors on failure.
    pub fn into_result(self) -> Result<(), Vec<ValidationError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
    }
}

/// Trait implemented by domain types that can validate themselves.
pub trait Validate {
    fn validate(&self) -> ValidationResult;
}

impl Validate for String {
    fn validate(&self) -> ValidationResult {
        if self.is_empty() {
            ValidationResult {
                errors: vec![ValidationError::Empty {
                    field: "value".into(),
                }],
            }
        } else {
            ValidationResult::ok()
        }
    }
}

impl Validate for str {
    fn validate(&self) -> ValidationResult {
        if self.is_empty() {
            ValidationResult {
                errors: vec![ValidationError::Empty {
                    field: "value".into(),
                }],
            }
        } else {
            ValidationResult::ok()
        }
    }
}

impl Validate for Money {
    fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::ok();
        if self.amount.is_sign_negative() {
            result.add(ValidationError::Negative {
                field: "amount".into(),
            });
        }
        if self.amount.is_sign_negative() || self.amount.is_zero() {
            result.add(ValidationError::NonPositive {
                field: "amount".into(),
            });
        }
        result
    }
}

/// Validate that a string is non-empty, trimmed length > 0.
pub fn validate_required(value: &str, field: &str) -> ValidationResult {
    if value.trim().is_empty() {
        ValidationResult {
            errors: vec![ValidationError::Empty {
                field: field.into(),
            }],
        }
    } else {
        ValidationResult::ok()
    }
}

/// Validate a maximum string length.
pub fn validate_length_max(value: &str, field: &str, max: usize) -> ValidationResult {
    let len = value.chars().count();
    if len > max {
        ValidationResult {
            errors: vec![ValidationError::TooLong {
                field: field.into(),
                max,
                got: len,
            }],
        }
    } else {
        ValidationResult::ok()
    }
}

/// Validate a currency code, returning the parsed [`Currency`] on success.
pub fn validate_currency(value: &str, field: &str) -> Result<Currency, ValidationError> {
    Currency::from_code(value).map_err(|_| ValidationError::InvalidCurrency {
        field: field.into(),
    })
}

/// Validate that a raw i64 can form a typed identifier of kind `$id`.
///
/// Use this at API boundaries before accepting client-supplied IDs (IDOR
/// prevention: an ID that fails validation is rejected, never coerced).
pub fn validate_id<I: TryFrom<i64, Error = DomainError>>(
    value: i64,
    field: &str,
) -> Result<I, ValidationError> {
    TryFrom::try_from(value).map_err(|_| ValidationError::InvalidId {
        field: field.into(),
    })
}

/// Validate an email address with a structural check (single `@`, non-empty
/// local and domain parts, domain contains a dot). Intentionaly dependency-free
/// (no `regex` crate) to keep the supply chain small; tighten later as needed.
pub fn validate_email(value: &str, field: &str) -> ValidationResult {
    let mut result = ValidationResult::ok();
    let at_count = value.matches('@').count();
    let valid = at_count == 1
        && value.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && !domain.is_empty() && domain.contains('.')
        });
    if !valid {
        result.add(ValidationError::InvalidEmail {
            field: field.into(),
        });
    }
    result
}

/// Validate a phone number: non-empty and contains only digits, spaces, `+`,
/// `-`, `(`, `)`.
pub fn validate_phone(value: &str, field: &str) -> ValidationResult {
    let mut result = ValidationResult::ok();
    if value.is_empty()
        || !value
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '+' | '-' | '(' | ')'))
    {
        result.add(ValidationError::InvalidPhone {
            field: field.into(),
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::{ClientId, LoanId};
    use proptest::prelude::*;

    #[test]
    fn empty_string_rejected() {
        let r = "".validate();
        assert!(!r.is_valid());
        assert_eq!(r.errors.len(), 1);
        assert!("abc".validate().is_valid());
    }

    #[test]
    fn money_validation() {
        let pos = Money::new(rust_decimal::Decimal::new(100, 2), Currency::USD);
        assert!(pos.validate().is_valid());
        let zero = Money::zero(Currency::USD);
        assert!(!zero.validate().is_valid());
        let neg = Money::new(rust_decimal::Decimal::new(-50, 2), Currency::USD);
        assert!(!neg.validate().is_valid());
        assert_eq!(neg.validate().errors.len(), 2); // negative + non-positive
    }

    #[test]
    fn required_and_length() {
        assert!(validate_required("  ", "name").errors.len() == 1);
        assert!(validate_required("Alice", "name").is_valid());
        assert!(validate_length_max("toolong", "field", 3).errors.len() == 1);
        assert!(validate_length_max("ok", "field", 10).is_valid());
    }

    #[test]
    fn currency_validation() {
        assert!(validate_currency("USD", "currency").is_ok());
        assert!(validate_currency("usd", "currency").is_err());
        assert!(validate_currency("US", "currency").is_err());
    }

    #[test]
    fn id_validation_rejects_non_positive() {
        assert!(validate_id::<LoanId>(0, "loan_id").is_err());
        assert!(validate_id::<LoanId>(-1, "loan_id").is_err());
        assert!(validate_id::<LoanId>(42, "loan_id").is_ok());
        // Different ID type with same value is not interchangeable.
        assert!(validate_id::<ClientId>(42, "client_id").is_ok());
    }

    #[test]
    fn email_validation() {
        assert!(validate_email("a@b.com", "email").is_valid());
        assert!(validate_email("not-an-email", "email").errors.len() == 1);
        assert!(validate_email("@b.com", "email").errors.len() == 1);
        assert!(validate_email("a@", "email").errors.len() == 1);
    }

    #[test]
    fn phone_validation() {
        assert!(validate_phone("+55 11 99999-9999", "phone").is_valid());
        assert!(validate_phone("", "phone").errors.len() == 1);
        assert!(validate_phone("abc!@#", "phone").errors.len() == 1);
    }

    #[test]
    fn validation_result_merge_and_into_result() {
        let mut a = ValidationResult::ok();
        a.add(ValidationError::Empty { field: "a".into() });
        let b = ValidationResult {
            errors: vec![ValidationError::Empty { field: "b".into() }],
        };
        a.merge(b);
        assert_eq!(a.errors.len(), 2);
        assert!(!a.is_valid());
        assert!(!a.clone().into_result().is_ok());
        assert!(ValidationResult::ok().into_result().is_ok());
    }

    #[test]
    fn proptest_currency_round_trip() {
        proptest!(|(code in "[A-Z]{3}")| {
            let c = validate_currency(&code, "currency");
            prop_assert!(c.is_ok());
            prop_assert_eq!(c.unwrap().code(), code);
        });
    }

    #[test]
    fn proptest_ids_round_trip() {
        proptest!(|(v in 1i64..1_000_000i64)| {
            let id = LoanId::new(v);
            prop_assert_eq!(id.value(), v);
            let parsed: LoanId = id.to_string().parse().unwrap();
            prop_assert_eq!(parsed, id);
        });
    }
}
