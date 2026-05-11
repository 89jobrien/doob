use anyhow::Result;

use crate::db::SqliteConnection;

pub fn initialize(conn: &SqliteConnection) -> Result<()> {
    conn.with_conn(|c| {
        c.execute_batch(
            "CREATE TABLE IF NOT EXISTS todo (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid        TEXT NOT NULL UNIQUE,
                content     TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'pending',
                priority    INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT,
                due_date    TEXT,
                project     TEXT,
                project_path TEXT,
                file_path   TEXT,
                tags        TEXT NOT NULL DEFAULT '[]',
                metadata    TEXT,
                blocks      TEXT NOT NULL DEFAULT '[]',
                blocked_by  TEXT NOT NULL DEFAULT '[]'
            );
            CREATE INDEX IF NOT EXISTS idx_todo_status ON todo(status);
            CREATE INDEX IF NOT EXISTS idx_todo_project ON todo(project);
            CREATE INDEX IF NOT EXISTS idx_todo_uuid ON todo(uuid);

            CREATE TABLE IF NOT EXISTS note (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid        TEXT NOT NULL UNIQUE,
                content     TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                project     TEXT,
                file_path   TEXT,
                tags        TEXT NOT NULL DEFAULT '[]',
                metadata    TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_note_project ON note(project);

            CREATE TABLE IF NOT EXISTS archive (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid         TEXT NOT NULL UNIQUE,
                content      TEXT NOT NULL,
                status       TEXT NOT NULL,
                priority     INTEGER NOT NULL DEFAULT 0,
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL,
                completed_at TEXT,
                due_date     TEXT,
                project      TEXT,
                project_path TEXT,
                file_path    TEXT,
                tags         TEXT NOT NULL DEFAULT '[]',
                blocks       TEXT NOT NULL DEFAULT '[]',
                blocked_by   TEXT NOT NULL DEFAULT '[]',
                archived_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_archive_project ON archive(project);

            CREATE TABLE IF NOT EXISTS handoff_item (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid         TEXT NOT NULL UNIQUE,
                handoff_id   TEXT NOT NULL UNIQUE,
                project      TEXT NOT NULL,
                title        TEXT NOT NULL,
                description  TEXT,
                priority     TEXT NOT NULL DEFAULT 'P2',
                status       TEXT NOT NULL DEFAULT 'open',
                files        TEXT NOT NULL DEFAULT '[]',
                extra        TEXT NOT NULL DEFAULT '[]',
                created_at   TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_handoff_project ON handoff_item(project);

            CREATE TABLE IF NOT EXISTS handoff_log (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                project    TEXT NOT NULL,
                date       TEXT NOT NULL,
                summary    TEXT NOT NULL,
                commits    TEXT NOT NULL DEFAULT '[]',
                created_at TEXT DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_handoff_log_project ON handoff_log(project);

            CREATE TABLE IF NOT EXISTS handoff_state (
                project    TEXT NOT NULL PRIMARY KEY,
                branch     TEXT,
                build      TEXT,
                tests      TEXT,
                notes      TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS metadata (
                key   TEXT NOT NULL PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        Ok(())
    })
}
