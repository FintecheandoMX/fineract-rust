//! Holiday repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Holiday, HolidayError};
use fineract_core::ids::HolidayId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type HolidayRepositoryResult<T> = Result<T, HolidayError>;

#[async_trait]
pub trait HolidayRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayRepositoryResult<Option<Holiday>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> HolidayRepositoryResult<Vec<Holiday>>;
    async fn find_by_date(
        &self,
        context: &AuthorizedContext,
        date: chrono::NaiveDate,
    ) -> HolidayRepositoryResult<Vec<Holiday>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        holiday: &Holiday,
    ) -> HolidayRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        holiday: &Holiday,
    ) -> HolidayRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayRepositoryResult<()>;
}

pub struct SqlxHolidayRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxHolidayRepository {
    pub fn new(
        pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        Self {
            pool_manager,
            connection_string,
        }
    }

    async fn pool_for_context(
        &self,
        context: &AuthorizedContext,
    ) -> HolidayRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl HolidayRepository for SqlxHolidayRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayRepositoryResult<Option<Holiday>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, holiday_date, is_recurring, active, created_at, updated_at FROM holidays WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| HolidayError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_holiday(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> HolidayRepositoryResult<Vec<Holiday>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, holiday_date, is_recurring, active, created_at, updated_at FROM holidays WHERE tenant_id = $1 ORDER BY holiday_date",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| HolidayError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_holiday(&r)).collect()
    }

    async fn find_by_date(
        &self,
        context: &AuthorizedContext,
        date: chrono::NaiveDate,
    ) -> HolidayRepositoryResult<Vec<Holiday>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, holiday_date, is_recurring, active, created_at, updated_at FROM holidays WHERE tenant_id = $1 AND (holiday_date = $2 OR (is_recurring AND EXTRACT(MONTH FROM holiday_date) = EXTRACT(MONTH FROM $2) AND EXTRACT(DAY FROM holiday_date) = EXTRACT(DAY FROM $2)))",
        )
        .bind(context.tenant_id.value())
        .bind(date)
        .fetch_all(&pool)
        .await
        .map_err(|e| HolidayError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_holiday(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        holiday: &Holiday,
    ) -> HolidayRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO holidays (id, tenant_id, name, holiday_date, is_recurring, active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(holiday.id.value())
        .bind(holiday.tenant_id.value())
        .bind(&holiday.name)
        .bind(holiday.holiday_date)
        .bind(holiday.is_recurring)
        .bind(holiday.active)
        .bind(holiday.created_at)
        .bind(holiday.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| HolidayError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        holiday: &Holiday,
    ) -> HolidayRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE holidays SET name = $2, holiday_date = $3, is_recurring = $4, active = $5, updated_at = $6 WHERE id = $1 AND tenant_id = $7",
        )
        .bind(holiday.id.value())
        .bind(&holiday.name)
        .bind(holiday.holiday_date)
        .bind(holiday.is_recurring)
        .bind(holiday.active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| HolidayError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(HolidayError::NotFound(holiday.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM holidays WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| HolidayError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(HolidayError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_holiday(row: &sqlx::postgres::PgRow) -> HolidayRepositoryResult<Holiday> {
    Ok(Holiday {
        id: HolidayId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            HolidayError::QueryError {
                detail: "invalid holiday id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            HolidayError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        holiday_date: row.try_get("holiday_date")?,
        is_recurring: row.try_get("is_recurring")?,
        active: row.try_get("active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    #![allow(async_fn_in_trait)]
    use super::*;
    use chrono::Datelike;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryHolidayRepository {
        store: Mutex<HashMap<(TenantId, HolidayId), Holiday>>,
    }

    impl Default for InMemoryHolidayRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryHolidayRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: HolidayId) -> (TenantId, HolidayId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl HolidayRepository for InMemoryHolidayRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: HolidayId,
        ) -> HolidayRepositoryResult<Option<Holiday>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> HolidayRepositoryResult<Vec<Holiday>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), h)| {
                    if *tid == context.tenant_id {
                        Some(h.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_by_date(
            &self,
            context: &AuthorizedContext,
            date: chrono::NaiveDate,
        ) -> HolidayRepositoryResult<Vec<Holiday>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), h)| {
                    if *tid == context.tenant_id
                        && (h.holiday_date == date
                            || (h.is_recurring
                                && h.holiday_date.month() == date.month()
                                && h.holiday_date.day() == date.day()))
                    {
                        Some(h.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            holiday: &Holiday,
        ) -> HolidayRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, holiday.id), holiday.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            holiday: &Holiday,
        ) -> HolidayRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, holiday.id)) {
                store.insert(Self::key(context.tenant_id, holiday.id), holiday.clone());
                Ok(())
            } else {
                Err(HolidayError::NotFound(holiday.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: HolidayId,
        ) -> HolidayRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(HolidayError::NotFound(id))
            }
        }
    }

    fn make_context(tenant_id: i64) -> AuthorizedContext {
        AuthorizedContext::new(
            TenantId::new(tenant_id),
            fineract_security::Principal {
                app_user_id: AppUserId::new(1),
                username: "tester".into(),
                roles: HashSet::new(),
                permissions: HashSet::new(),
                office_id: None,
            },
            SessionId::new(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn holiday_repository_is_tenant_isolated() {
        let repo = InMemoryHolidayRepository::new();
        let holiday = Holiday::new(
            HolidayId::new(42),
            TenantId::new(1),
            "Christmas".to_string(),
            chrono::NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
            true,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &holiday).await.unwrap();
        let result = repo.find_by_id(&ctx, HolidayId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Christmas");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, HolidayId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
