use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct CacheDb {
    conn: Connection,
}

impl CacheDb {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self> {
        let db_path = app_data_dir.join("cache.db");
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    // For testing
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS repos (
                id INTEGER PRIMARY KEY,
                instance_id TEXT NOT NULL,
                data TEXT NOT NULL,
                cached_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS issues (
                id INTEGER PRIMARY KEY,
                instance_id TEXT NOT NULL,
                repo_full_name TEXT NOT NULL,
                data TEXT NOT NULL,
                cached_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS pull_requests (
                id INTEGER PRIMARY KEY,
                instance_id TEXT NOT NULL,
                repo_full_name TEXT NOT NULL,
                data TEXT NOT NULL,
                cached_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_issues_repo
                ON issues(instance_id, repo_full_name);
            CREATE INDEX IF NOT EXISTS idx_pulls_repo
                ON pull_requests(instance_id, repo_full_name);"
        )?;
        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_run_cleanly() {
        let db = CacheDb::in_memory().expect("Failed to create in-memory db");
        // Running migrations again should be idempotent
        db.run_migrations().expect("Second migration run failed");
    }

    #[test]
    fn test_tables_exist() {
        let db = CacheDb::in_memory().expect("Failed to create in-memory db");

        let count: i64 = db.connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('repos', 'issues', 'pull_requests')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query tables");

        assert_eq!(count, 3);
    }
}
