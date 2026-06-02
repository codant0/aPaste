use rusqlite::{Connection, Result, params};
use super::manager::ClipboardItem;

pub fn search(conn: &Connection, query: &str, limit: i64) -> Result<Vec<ClipboardItem>> {
    let escaped = escape_fts5(query);
    let fts_query = if escaped.is_empty() {
        return get_all(conn, limit);
    } else {
        format!("{}*", escaped)
    };

    let mut stmt = conn.prepare(
        "SELECT ci.id, ci.content, ci.source_app, ci.created_at, ci.last_used_at, ci.is_favorite
         FROM clipboard_items ci
         INNER JOIN clipboard_fts fts ON ci.id = fts.rowid
         WHERE clipboard_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    )?;

    let items = stmt.query_map(params![fts_query, limit], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            source_app: row.get(2)?,
            created_at: row.get(3)?,
            last_used_at: row.get(4)?,
            is_favorite: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

fn get_all(conn: &Connection, limit: i64) -> Result<Vec<ClipboardItem>> {
    super::manager::get_recent(conn, limit, 0)
}

pub fn search_favorites(conn: &Connection, query: &str, limit: i64) -> Result<Vec<ClipboardItem>> {
    let escaped = escape_fts5(query);
    if escaped.is_empty() {
        return super::manager::get_favorites(conn, limit, 0);
    }

    let fts_query = format!("{}*", escaped);
    let mut stmt = conn.prepare(
        "SELECT ci.id, ci.content, ci.source_app, ci.created_at, ci.last_used_at, ci.is_favorite
         FROM clipboard_items ci
         INNER JOIN clipboard_fts fts ON ci.id = fts.rowid
         WHERE clipboard_fts MATCH ?1 AND ci.is_favorite = 1
         ORDER BY rank
         LIMIT ?2"
    )?;

    let items = stmt.query_map(params![fts_query, limit], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            source_app: row.get(2)?,
            created_at: row.get(3)?,
            last_used_at: row.get(4)?,
            is_favorite: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

fn escape_fts5(query: &str) -> String {
    let special = ['*', '"', '(', ')', '-', ':', '^'];
    let trimmed = query.trim();
    let mut result = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if special.contains(&ch) {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;
    use crate::history::manager::{add_item, toggle_favorite};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate::run(&conn).unwrap();
        add_item(&conn, "import React from 'react'", "h1", None).unwrap();
        add_item(&conn, "npm install tauri", "h2", None).unwrap();
        add_item(&conn, "const x = 42", "h3", None).unwrap();
        add_item(&conn, "React hooks are useful", "h4", None).unwrap();
        conn
    }

    #[test]
    fn test_fuzzy_search() {
        let conn = setup();
        let results = search(&conn, "react", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_no_results() {
        let conn = setup();
        let results = search(&conn, "zzzznotfound", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_query_returns_all() {
        let conn = setup();
        let results = search(&conn, "", 10).unwrap();
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_search_favorites_filtered() {
        let conn = setup();
        // Mark items 1 and 3 as favorites
        toggle_favorite(&conn, 1).unwrap();
        toggle_favorite(&conn, 3).unwrap();

        // Search within favorites
        let results = search_favorites(&conn, "react", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);

        // Non-favorite search term returns nothing
        let results = search_favorites(&conn, "tauri", 10).unwrap();
        assert_eq!(results.len(), 0);

        // Empty query returns all favorites
        let results = search_favorites(&conn, "", 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|i| i.is_favorite));
    }
}
