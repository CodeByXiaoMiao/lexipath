use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use rusqlite::{params, Connection};

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn open_portable() -> anyhow::Result<Self> {
        let data_dir = portable_data_dir()?;
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("failed to create data directory {}", data_dir.display()))?;

        let connection = Connection::open(data_dir.join("lexipath.db"))
            .context("failed to open LexiPath database")?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS lesson_progress (
                 lesson_id TEXT PRIMARY KEY,
                 completed INTEGER NOT NULL,
                 first_attempt_accuracy REAL NOT NULL,
                 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );",
        )?;

        Ok(Self { connection })
    }

    pub fn save_lesson_complete(
        &self,
        lesson_id: &str,
        first_attempt_accuracy: f32,
    ) -> anyhow::Result<()> {
        self.connection.execute(
            "INSERT INTO lesson_progress (lesson_id, completed, first_attempt_accuracy)
             VALUES (?1, 1, ?2)
             ON CONFLICT(lesson_id) DO UPDATE SET
                 completed = 1,
                 first_attempt_accuracy = excluded.first_attempt_accuracy,
                 updated_at = CURRENT_TIMESTAMP",
            params![lesson_id, first_attempt_accuracy],
        )?;
        Ok(())
    }
}

fn portable_data_dir() -> anyhow::Result<PathBuf> {
    let executable = std::env::current_exe().context("failed to locate executable")?;
    let parent = executable
        .parent()
        .context("executable has no parent directory")?;
    Ok(parent.join("data"))
}
