//! Database utilities and connection management

use sqlx::{PgPool, Row};
use uuid::Uuid;
use anyhow::Result;

/// Database connection and query utilities
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    /// Get database version
    pub async fn get_version(&self) -> Result<String> {
        let row = sqlx::query("SELECT version()")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get(0))
    }
}

/// Database transaction helper
pub async fn with_transaction<F, R>(pool: &PgPool, f: F) -> Result<R>
where
    F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<R>,
{
    let mut tx = pool.begin().await?;
    let result = f(&mut tx)?;
    tx.commit().await?;
    Ok(result)
}

/// Query builder helpers
pub mod queries {
    use super::*;

    /// Build a paginated query
    pub fn paginate(base_query: &str, page: u32, limit: u32) -> String {
        let offset = (page - 1) * limit;
        format!("{} LIMIT {} OFFSET {}", base_query, limit, offset)
    }

    /// Count total records for a query
    pub fn count_query(base_query: &str) -> String {
        format!("SELECT COUNT(*) as total FROM ({}) as subquery", base_query)
    }
}

/// Database initialization and seeding
pub mod migrations {
    use super::*;

    /// Initialize database with required data
    pub async fn initialize_database(pool: &PgPool) -> Result<()> {
        // Run migrations is handled by sqlx::migrate! in main
        
        // Check if we have basic data
        let staff_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM staff")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        if staff_count == 0 {
            tracing::info!("Database appears empty, running initial setup");
            // Initial data is inserted via migration files
        }

        Ok(())
    }
}
