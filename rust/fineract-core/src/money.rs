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

//! Monetary amount value object.
//!
//! Amounts are stored as `rust_decimal::Decimal` to avoid floating-point
//! rounding errors. A `Currency` (ISO 4217, 3 uppercase letters) is part of
//! the type so amounts of different currencies cannot be accidentally mixed.

use std::fmt;
use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ISO 4217 currency code, validated at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Currency(pub [u8; 3]);

impl Currency {
    pub const USD: Currency = Currency(*b"USD");
    pub const MXN: Currency = Currency(*b"MXN");
    pub const EUR: Currency = Currency(*b"EUR");

    /// Parse a 3-letter uppercase currency code. Returns an error for anything
    /// else (rejects lowercase, symbols, wrong length).
    pub fn from_code(code: &str) -> Result<Self, CurrencyError> {
        let bytes: [u8; 3] = code
            .as_bytes()
            .try_into()
            .map_err(|_| CurrencyError::InvalidLength)?;
        for b in &bytes {
            if !b.is_ascii_uppercase() {
                return Err(CurrencyError::InvalidCharacter(*b as char));
            }
        }
        Ok(Currency(bytes))
    }

    pub fn code(self) -> String {
        String::from_utf8_lossy(&self.0).to_string()
    }
}

#[derive(Debug, Clone, Error)]
pub enum CurrencyError {
    #[error("currency must be exactly 3 uppercase ASCII letters")]
    InvalidLength,
    #[error("currency contains invalid character '{0}'")]
    InvalidCharacter(char),
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = CurrencyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_code(s)
    }
}

/// A monetary amount paired with a currency. Immutable and compared by value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Zero amount for a currency.
    pub fn zero(currency: Currency) -> Self {
        Self::new(Decimal::ZERO, currency)
    }

    /// Returns true when the amount is strictly positive.
    pub fn is_positive(self) -> bool {
        self.amount > Decimal::ZERO
    }

    /// Returns true when the amount is zero or negative (invalid for posting).
    pub fn is_non_positive(self) -> bool {
        self.amount <= Decimal::ZERO
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}
