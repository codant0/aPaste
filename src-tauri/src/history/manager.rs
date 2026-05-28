use rusqlite::{Connection, Result, params};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub source_app: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

pub fn add_item(
    conn: &Connection,
    content: &str,
    content_hash: &str,
    source_app: Option<&str>,
) -> Result<()> {
    // Check for duplicate — if last item has same hash, skip
    let last_hash: Option<String> = conn
        .query_row(
            "SELECT content_hash FROM clipboard_items ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();

    if last_hash.as_deref() == Some(content_hash) {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO clipboard_items (content, content_hash, source_app) VALUES (?1, ?2, ?3)",
        params![content, content_hash, source_app],
    )?;

    Ok(())
}

pub fn get_recent(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<ClipboardItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, source_app, created_at, last_used_at
         FROM clipboard_items
         ORDER BY id DESC
         LIMIT ?1 OFFSET ?2"
    )?;

    let items = stmt.query_map(params![limit, offset], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            source_app: row.get(2)?,
            created_at: row.get(3)?,
            last_used_at: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_all(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items", [])?;
    conn.execute("DELETE FROM clipboard_fts", [])?;
    Ok(())
}

pub fn update_last_used(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clipboard_items SET last_used_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate::run(&conn).unwrap();
        conn
    }

    #[test]
    fn test_add_and_get_recent() {
        let conn = setup_db();
        add_item(&conn, "hello world", "hash1", Some("Notepad")).unwrap();
        add_item(&conn, "foo bar", "hash2", None).unwrap();

        let items = get_recent(&conn, 10, 0).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "foo bar"); // Most recent first
    }

    #[test]
    fn test_dedup_skips_consecutive_same_hash() {
        let conn = setup_db();
        add_item(&conn, "hello", "hash1", None).unwrap();
        add_item(&conn, "hello again", "hash1", None).unwrap(); // Same hash as last

        let items = get_recent(&conn, 10, 0).unwrap();
        assert_eq!(items.len(), 1); // Second insert skipped
    }

    #[test]
    fn test_delete_and_clear() {
        let conn = setup_db();
        add_item(&conn, "a", "h1", None).unwrap();
        add_item(&conn, "b", "h2", None).unwrap();

        delete_item(&conn, 2).unwrap();
        assert_eq!(get_recent(&conn, 10, 0).unwrap().len(), 1);

        clear_all(&conn).unwrap();
        assert_eq!(get_recent(&conn, 10, 0).unwrap().len(), 0);
    }
}
