//! Base repository traits and Unit-of-Work patterns.
//!
//! These are pure traits (no sqlx coupling) so the domain layer can depend on
//! them without pulling in the persistence crate. Implementations live in
//! [`fineract-persistence`].

use crate::ids::TenantId;
use std::error::Error;

/// A generic repository over an entity type `Entity` keyed by `Id`.
///
/// All implementations must enforce tenant filtering by default where
/// applicable; see [`TenantRepository`].
pub trait Repository<Entity, Id>
where
    Entity: Send + Sync,
    Id: Send + Sync,
{
    type Error: Error + Send + Sync;

    async fn find_by_id(&self, id: Id) -> Result<Option<Entity>, Self::Error>;
    async fn save(&self, entity: &Entity) -> Result<(), Self::Error>;
    async fn delete(&self, id: Id) -> Result<(), Self::Error>;
}

/// A repository that additionally scopes reads to a single tenant.
///
/// Tenant isolation is enforced here by construction: every read takes an
/// explicit `TenantId` and must never return data from another tenant.
pub trait TenantRepository<Entity, Id>: Repository<Entity, Id>
where
    Entity: Send + Sync,
    Id: Send + Sync,
{
    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<Entity>, Self::Error>;
}

/// A unit of work that groups multiple repository mutations into a single
/// transactional boundary (commit / rollback).
pub trait UnitOfWork {
    type Error: Error + Send + Sync;

    async fn commit(&mut self) -> Result<(), Self::Error>;
    async fn rollback(&mut self) -> Result<(), Self::Error>;
}
