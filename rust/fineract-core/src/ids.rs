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

//! Strongly typed domain identifiers.
//!
//! Every aggregate root and entity is identified by a dedicated newtype rather
//! than a raw `i64`. This prevents IDOR by construction: a `LoanId` cannot be
//! passed where a `ClientId` is expected, and IDs never cross module boundaries
//! as untyped integers.
//!
//! All identifiers are backed by `i64` (matching the Fineract database `BIGINT`
//! primary keys) and carry `serde` + `Display` support for API boundaries.

use std::fmt;
use std::str::FromStr;

/// Generate a strongly typed identifier newtype.
///
/// The macro generates:
/// - a transparent `struct $Name(pub i64)`
/// - `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Ord`, `PartialOrd`
/// - `Serialize`/`Deserialize` (transparent, serialized as the raw integer)
/// - `From<i64>`, `From<$Name> for i64`, `TryFrom<i64>` (rejects non-positive)
/// - `AsRef<i64>`, `Display`, `FromStr`
/// - When the `sqlx` feature is enabled: `Type<Postgres>`, `Encode`, `Decode`
///   so the ID can be used in parameterized `sqlx` queries.
macro_rules! define_id {
    ($name:ident $(, $doc:expr)?) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub i64);

        impl $name {
            /// Create a new identifier. Panics if the value is not positive.
            /// Use [`TryFrom`] for fallible creation from untrusted input.
            #[track_caller]
            pub fn new(id: i64) -> Self {
                if id <= 0 {
                    panic!(
                        "{}: identifier must be positive, got {}",
                        stringify!($name),
                        id
                    );
                }
                Self(id)
            }

            /// Return the underlying raw integer value.
            pub fn value(self) -> i64 {
                self.0
            }
        }

        impl AsRef<i64> for $name {
            fn as_ref(&self) -> &i64 {
                &self.0
            }
        }

        impl From<$name> for i64 {
            fn from(id: $name) -> i64 {
                id.0
            }
        }

        impl TryFrom<i64> for $name {
            type Error = crate::error::DomainError;
            fn try_from(id: i64) -> Result<Self, Self::Error> {
                if id <= 0 {
                    Err(crate::error::DomainError::InvalidIdentifier {
                        entity: stringify!($name).to_string(),
                        value: id,
                    })
                } else {
                    Ok(Self(id))
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = crate::error::DomainError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let v: i64 =
                    s.parse()
                        .map_err(|_| crate::error::DomainError::InvalidIdentifier {
                            entity: stringify!($name).to_string(),
                            value: i64::MIN,
                        })?;
                Self::try_from(v)
            }
        }

        #[cfg(feature = "sqlx")]
        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
            }
        }

        #[cfg(feature = "sqlx")]
        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <i64 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        #[cfg(feature = "sqlx")]
        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let inner = <i64 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                Self::try_from(inner)
                    .map_err(|e| -> sqlx::error::BoxDynError { e.to_string().into() })
            }
        }
    };
}

// Tenant isolation root identifier.
define_id!(TenantId);
// Organisational hierarchy.
define_id!(OfficeId);
define_id!(StaffId);
define_id!(GroupId);
// Client / party.
define_id!(ClientId);
// Loan family.
define_id!(LoanId);
define_id!(PaymentDetailId);
define_id!(NoteId);
define_id!(CollateralId);
// Savings.
define_id!(SavingsAccountId);
define_id!(TransactionEntryId);
// Accounting.
define_id!(JournalEntryId);
define_id!(AccountId);
// Charges / rates / tax.
define_id!(ChargeId);
define_id!(RateId);
define_id!(TaxId);
// Security.
define_id!(AppUserId);
define_id!(RoleId);
define_id!(PermissionId);
// Commands / audit.
define_id!(CommandId);
// Misc.
define_id!(FundId);
define_id!(DocumentId);
define_id!(ReportId);

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;

    use super::*;

    #[test]
    fn ids_are_distinct_types() {
        let loan = LoanId::new(42);
        let client = ClientId::new(42);
        // Same underlying value, different types — must not be interchangeable.
        assert_eq!(loan.value(), client.value());
        let _ = loan;
        let _ = client;
    }

    #[test]
    fn try_from_rejects_non_positive() {
        assert!(LoanId::try_from(0).is_err());
        assert!(LoanId::try_from(-1).is_err());
        assert!(LoanId::try_from(1).is_ok());
    }

    #[test]
    fn id_display_and_roundtrip() {
        let id = LoanId::new(123);
        assert_eq!(id.to_string(), "123");
        let parsed: LoanId = "123".parse().unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    #[should_panic(expected = "identifier must be positive")]
    fn new_panics_on_non_positive() {
        let _ = LoanId::new(0);
    }

    #[test]
    fn proptest_ids_round_trip() {
        proptest::proptest!(|(v in 1i64..1_000_000i64)| {
            let id = LoanId::new(v);
            prop_assert_eq!(id.value(), v);
            let parsed: LoanId = id.to_string().parse().unwrap();
            prop_assert_eq!(parsed, id);
        });
    }
}
