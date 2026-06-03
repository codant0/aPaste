use rusqlite::{Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clipboard_items (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            content      TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            source_app   TEXT,
            image        BLOB,
            created_at   TEXT NOT NULL DEFAULT (datetime('now')),
            last_used_at TEXT
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts
        USING fts5(content, content='clipboard_items', content_rowid='id');

        CREATE TRIGGER IF NOT EXISTS clipboard_ai
        AFTER INSERT ON clipboard_items BEGIN
            INSERT INTO clipboard_fts(rowid, content)
            VALUES (new.id, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS clipboard_ad
        AFTER DELETE ON clipboard_items BEGIN
            INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
            VALUES ('delete', old.id, old.content);
        END;

        CREATE TRIGGER IF NOT EXISTS clipboard_au
        AFTER UPDATE ON clipboard_items BEGIN
            INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
            VALUES ('delete', old.id, old.content);
            INSERT INTO clipboard_fts(rowid, content)
            VALUES (new.id, new.content);
        END;

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO settings (key, value) VALUES ('max_items', '1000');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('max_days', '30');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('hotkey', 'Win+Shift+V');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'true');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'light');"
    )?;

    // Add is_favorite column if it doesn't exist
    let has_column: bool = conn
        .prepare("PRAGMA table_info(clipboard_items)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|name| name == "is_favorite");

    if !has_column {
        conn.execute(
            "ALTER TABLE clipboard_items ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    // Index for global dedup queries
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_content_hash ON clipboard_items(content_hash);"
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 5);

        // Verify FTS table exists
        conn.execute("INSERT INTO clipboard_items (content, content_hash) VALUES ('test', 'abc')", [])
            .unwrap();
        let fts_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM clipboard_fts WHERE clipboard_fts MATCH 'test'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fts_count, 1);

        // Verify is_favorite column exists and defaults to 0
        conn.execute("INSERT INTO clipboard_items (content, content_hash) VALUES ('fav-test', 'xyz')", [])
            .unwrap();
        let is_fav: bool = conn
            .query_row("SELECT is_favorite FROM clipboard_items WHERE content_hash = 'xyz'", [], |r| r.get(0))
            .unwrap();
        assert!(!is_fav);
    }
}
