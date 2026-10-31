//! Domain model types used by the persistence layer.
//!
//! These are lightweight representations of database rows used for
//! repository implementations. They mirror the core domain types but are
//! kept separate to avoid circular dependencies.

use serde::{Deserialize, Serialize};

use fineract_core::ids::*;
use fineract_core::money::Money;

/// A client record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRow {
    pub id: ClientId,
    pub tenant_id: TenantId,
    pub office_id: Option<OfficeId>,
    pub display_name: String,
    pub email: Option<String>,
    pub active: bool,
}

/// A loan record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanRow {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub client_id: ClientId,
    pub office_id: Option<OfficeId>,
    pub principal: Money,
    pub interest_rate: f64,
    pub status: String,
}

/// A savings account record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsAccountRow {
    pub id: SavingsAccountId,
    pub tenant_id: TenantId,
    pub client_id: ClientId,
    pub office_id: Option<OfficeId>,
    pub balance: Money,
    pub status: String,
}
