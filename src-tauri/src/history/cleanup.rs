use rusqlite::{Connection, Result, params};

pub fn run_cleanup(conn: &Connection, max_items: i64, max_days: i64) -> Result<usize> {
    let mut deleted = 0;

    // Delete by age (exclude favorites)
    deleted += conn.execute(
        "DELETE FROM clipboard_items
         WHERE created_at < datetime('now', ?1)
         AND is_favorite = 0",
        params![format!("-{} days", max_days)],
    )?;

    // Delete by count — keep only max_items most recent (exclude favorites)
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM clipboard_items WHERE is_favorite = 0", [], |r| r.get(0)
    )?;

    if total > max_items {
        let to_delete = total - max_items;
        conn.execute(
            "DELETE FROM clipboard_items WHERE id IN (
                SELECT id FROM clipboard_items WHERE is_favorite = 0 ORDER BY id ASC LIMIT ?1
            )",
            params![to_delete],
        )?;
        deleted += to_delete as usize;
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_cleanup_by_count() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE clipboard_items (id INTEGER PRIMARY KEY, content TEXT, created_at TEXT DEFAULT (datetime('now')), is_favorite INTEGER NOT NULL DEFAULT 0);
             INSERT INTO clipboard_items (content) VALUES ('a'), ('b'), ('c'), ('d'), ('e');"
        ).unwrap();

        let deleted = run_cleanup(&conn, 3, 365).unwrap();
        assert!(deleted >= 2);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_cleanup_preserves_favorites() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE clipboard_items (id INTEGER PRIMARY KEY, content TEXT, created_at TEXT DEFAULT (datetime('now')), is_favorite INTEGER NOT NULL DEFAULT 0);
             INSERT INTO clipboard_items (content, is_favorite) VALUES ('fav1', 1), ('fav2', 1), ('old1', 0), ('old2', 0), ('old3', 0);"
        ).unwrap();

        // max_items=2 applies only to non-favorites. 3 non-fav → keep 2 + 2 favs = 4 total
        let deleted = run_cleanup(&conn, 2, 365).unwrap();
        assert_eq!(deleted, 1); // deleted 1 non-favorite
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 4); // 2 favorites + 2 non-favorites kept
        let fav_count: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_items WHERE is_favorite = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(fav_count, 2);
    }
}
