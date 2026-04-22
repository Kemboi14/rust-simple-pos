//! Database utilities and connection management

use sqlx::{PgPool, Row};
use anyhow::Result;

/// Database connection and query utilities
#[allow(dead_code)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get the connection pool
    #[allow(dead_code)]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Test database connection
    #[allow(dead_code)]
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    /// Get database version
    #[allow(dead_code)]
    pub async fn get_version(&self) -> Result<String> {
        let row = sqlx::query("SELECT version()")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get(0))
    }
}

/// Database transaction helper
#[allow(dead_code)]
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
    /// Build a paginated query
    #[allow(dead_code)]
    pub fn paginate(base_query: &str, page: u32, limit: u32) -> String {
        let offset = (page - 1) * limit;
        format!("{} LIMIT {} OFFSET {}", base_query, limit, offset)
    }

    /// Count total records for a query
    #[allow(dead_code)]
    pub fn count_query(base_query: &str) -> String {
        format!("SELECT COUNT(*) as total FROM ({}) as subquery", base_query)
    }
}

/// Database initialization and seeding
pub mod migrations {
    use sqlx::PgPool;
    use anyhow::Result;

    /// Initialize database with required data
    #[allow(dead_code)]
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
