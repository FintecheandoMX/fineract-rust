//! Domain-level enumerations shared across Fineract modules.
//!
//! These enums encode state and classification in the type system rather than
//! magic strings/numbers, and are serialized/deserialized for API boundaries.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Error returned when parsing a string into a domain enum fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("failed to parse '{value}' as {target}")]
pub struct ParseEnumError {
    pub target: String,
    pub value: String,
}

/// Direction of a transaction posting (debit / credit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionEntryType {
    Debit,
    Credit,
}

impl fmt::Display for TransactionEntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debit => write!(f, "DEBIT"),
            Self::Credit => write!(f, "CREDIT"),
        }
    }
}

impl FromStr for TransactionEntryType {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBIT" => Ok(Self::Debit),
            "CREDIT" => Ok(Self::Credit),
            _ => Err(ParseEnumError {
                target: "TransactionEntryType".into(),
                value: s.into(),
            }),
        }
    }
}

/// High-level accounting account classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Asset => write!(f, "ASSET"),
            Self::Liability => write!(f, "LIABILITY"),
            Self::Equity => write!(f, "EQUITY"),
            Self::Income => write!(f, "INCOME"),
            Self::Expense => write!(f, "EXPENSE"),
        }
    }
}

impl FromStr for AccountType {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ASSET" => Ok(Self::Asset),
            "LIABILITY" => Ok(Self::Liability),
            "EQUITY" => Ok(Self::Equity),
            "INCOME" => Ok(Self::Income),
            "EXPENSE" => Ok(Self::Expense),
            _ => Err(ParseEnumError {
                target: "AccountType".into(),
                value: s.into(),
            }),
        }
    }
}

/// Lifecycle state of a command / maker-checker request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandState {
    Pending,
    Approved,
    Rejected,
    Executed,
    Expired,
}

impl fmt::Display for CommandState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Approved => write!(f, "APPROVED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Executed => write!(f, "EXECUTED"),
            Self::Expired => write!(f, "EXPIRED"),
        }
    }
}

impl FromStr for CommandState {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PENDING" => Ok(Self::Pending),
            "APPROVED" => Ok(Self::Approved),
            "REJECTED" => Ok(Self::Rejected),
            "EXECUTED" => Ok(Self::Executed),
            "EXPIRED" => Ok(Self::Expired),
            _ => Err(ParseEnumError {
                target: "CommandState".into(),
                value: s.into(),
            }),
        }
    }
}

/// Interest calculation basis for loans / savings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterestType {
    Flat,
    ReducingBalance,
}

impl fmt::Display for InterestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flat => write!(f, "FLAT"),
            Self::ReducingBalance => write!(f, "REDUCING_BALANCE"),
        }
    }
}

impl FromStr for InterestType {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "FLAT" => Ok(Self::Flat),
            "REDUCING_BALANCE" => Ok(Self::ReducingBalance),
            _ => Err(ParseEnumError {
                target: "InterestType".into(),
                value: s.into(),
            }),
        }
    }
}
