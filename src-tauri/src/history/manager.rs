use rusqlite::{Connection, Result, params};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub source_app: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub is_favorite: bool,
    pub favorite_name: Option<String>,
}

pub fn add_item(
    conn: &Connection,
    content: &str,
    content_hash: &str,
    source_app: Option<&str>,
) -> Result<()> {
    // Check if content already exists anywhere in the database
    if let Ok(existing_id) = conn.query_row(
        "SELECT id FROM clipboard_items WHERE content_hash = ?1",
        params![content_hash],
        |r| r.get::<_, i64>(0),
    ) {
        // Already exists: update timestamps to bring it to the top
        conn.execute(
            "UPDATE clipboard_items SET created_at = datetime('now'), last_used_at = datetime('now') WHERE id = ?1",
            params![existing_id],
        )?;
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
        "SELECT id, content, source_app, created_at, last_used_at, is_favorite, favorite_name
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
            is_favorite: row.get(5)?,
            favorite_name: row.get(6)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_all(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items WHERE is_favorite = 0", [])?;
    Ok(())
}

pub fn toggle_favorite(conn: &Connection, id: i64) -> Result<bool> {
    conn.execute(
        "UPDATE clipboard_items SET is_favorite = NOT is_favorite WHERE id = ?1",
        params![id],
    )?;
    conn.query_row(
        "SELECT is_favorite FROM clipboard_items WHERE id = ?1",
        params![id],
        |r| r.get(0),
    )
}

pub fn get_favorites(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<ClipboardItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, source_app, created_at, last_used_at, is_favorite, favorite_name
         FROM clipboard_items
         WHERE is_favorite = 1
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
            is_favorite: row.get(5)?,
            favorite_name: row.get(6)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

pub fn update_last_used(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clipboard_items SET last_used_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn rename_favorite(conn: &Connection, id: i64, name: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE clipboard_items SET favorite_name = ?1 WHERE id = ?2 AND is_favorite = 1",
        params![name, id],
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

    fn fav_item(conn: &Connection, content: &str, hash: &str, fav: bool) {
        add_item(conn, content, hash, None).unwrap();
        if fav {
            let id: i64 = conn
                .query_row("SELECT id FROM clipboard_items WHERE content_hash = ?1", params![hash], |r| r.get(0))
                .unwrap();
            toggle_favorite(conn, id).unwrap();
        }
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = setup_db();
        add_item(&conn, "fav me", "hf1", None).unwrap();
        let id: i64 = conn
            .query_row("SELECT id FROM clipboard_items WHERE content_hash = 'hf1'", [], |r| r.get(0))
            .unwrap();

        // Toggle on
        let new_state = toggle_favorite(&conn, id).unwrap();
        assert!(new_state);

        // Toggle off
        let new_state = toggle_favorite(&conn, id).unwrap();
        assert!(!new_state);
    }

    #[test]
    fn test_get_favorites() {
        let conn = setup_db();
        fav_item(&conn, "fav1", "hf2", true);
        fav_item(&conn, "normal", "hf3", false);
        fav_item(&conn, "fav2", "hf4", true);

        let favs = get_favorites(&conn, 10, 0).unwrap();
        assert_eq!(favs.len(), 2);
        assert!(favs.iter().all(|i| i.is_favorite));
        // Most recent first
        assert_eq!(favs[0].content, "fav2");
        assert_eq!(favs[1].content, "fav1");
    }

    #[test]
    fn test_clear_all_preserves_favorites() {
        let conn = setup_db();
        fav_item(&conn, "keep", "hk1", true);
        fav_item(&conn, "delete-me", "hk2", false);
        fav_item(&conn, "also-keep", "hk3", true);

        clear_all(&conn).unwrap();

        let remaining = get_recent(&conn, 10, 0).unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.iter().all(|i| i.is_favorite));
    }
}
