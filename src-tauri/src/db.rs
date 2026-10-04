use rusqlite::{Connection, Result};

pub fn open_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE VIRTUAL TABLE files_fts USING fts5(name, path);",
    )?;
    Ok(conn)
}

pub fn search(conn: &Connection, prefix: &str) -> Result<Vec<String>> {
    // The trailing * makes it a prefix search: "rep*" matches "report".
    let query = format!("{prefix}*");
    let mut stmt = conn.prepare(
        "SELECT name FROM files_fts WHERE files_fts MATCH ?1 ORDER BY rank",
    )?;
    let rows = stmt.query_map([query], |row| row.get(0))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_search_finds_report() {
        let conn = open_memory().unwrap();
        for (name, path) in [
            ("report.pdf", "C:/docs/report.pdf"),
            ("notes.txt", "C:/docs/notes.txt"),
            ("photo.png", "C:/pics/photo.png"),
        ] {
            conn.execute(
                "INSERT INTO files_fts (name, path) VALUES (?1, ?2)",
                [name, path],
            )
            .unwrap();
        }
        assert_eq!(search(&conn, "rep").unwrap(), vec!["report.pdf"]);
    }
}