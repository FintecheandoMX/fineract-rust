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
//! invalid input early; never coerce or silently drop invalid values.

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
    #[error("{0}")]
    Custom(String),
}

/// Trait implemented by domain types that can validate themselves.
pub trait Validate {
    type Error;
    fn validate(&self) -> Result<(), Self::Error>;
}

impl Validate for String {
    type Error = ValidationError;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.is_empty() {
            Err(ValidationError::Empty {
                field: "value".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl Validate for str {
    type Error = ValidationError;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.is_empty() {
            Err(ValidationError::Empty {
                field: "value".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl Validate for Money {
    type Error = ValidationError;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.amount.is_sign_negative() {
            return Err(ValidationError::Negative {
                field: "amount".into(),
            });
        }
        if self.amount.is_zero() {
            return Err(ValidationError::NonPositive {
                field: "amount".into(),
            });
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::LoanId;
    use proptest::prelude::*;

    #[test]
    fn empty_string_rejected() {
        assert!("".validate().is_err());
        assert!("abc".validate().is_ok());
    }

    #[test]
    fn money_must_be_positive() {
        let pos = Money::new(rust_decimal::Decimal::new(100, 2), Currency::USD);
        assert!(pos.validate().is_ok());
        let zero = Money::zero(Currency::USD);
        assert!(zero.validate().is_err());
        let neg = Money::new(rust_decimal::Decimal::new(-50, 2), Currency::USD);
        assert!(neg.validate().is_err());
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
    }

    #[test]
    fn proptest_currency_round_trip() {
        proptest!(|(code in "[A-Z]{3}")| {
            let c = validate_currency(&code, "currency");
            prop_assert!(c.is_ok());
            prop_assert_eq!(c.unwrap().code(), code);
        });
    }
}
