use std::collections::HashMap;

use serde::Deserialize;
use worker::{SqlStorage, SqlStorageValue};

#[derive(Deserialize)]
struct Row {
    page: String,
    count: i64,
}

fn rows_to_counts(rows: Vec<Row>) -> HashMap<String, u64> {
    rows.into_iter()
        .map(|r| (r.page, r.count.max(0) as u64))
        .collect()
}

pub fn init_schema(sql: &SqlStorage) -> worker::Result<()> {
    sql.exec(
        "CREATE TABLE IF NOT EXISTS page_views (
            page  TEXT PRIMARY KEY,
            count INTEGER NOT NULL DEFAULT 0
        )",
        None,
    )?;
    Ok(())
}

pub fn load_counts(sql: &SqlStorage) -> worker::Result<HashMap<String, u64>> {
    let rows: Vec<Row> = sql
        .exec("SELECT page, count FROM page_views", None)?
        .to_array()?;
    Ok(rows_to_counts(rows))
}

pub fn flush_counts(sql: &SqlStorage, counts: &HashMap<String, u64>) -> worker::Result<()> {
    for (page, &count) in counts {
        sql.exec(
            "INSERT INTO page_views (page, count) VALUES (?, ?)
             ON CONFLICT(page) DO UPDATE SET count = excluded.count",
            vec![
                SqlStorageValue::String(page.clone()),
                SqlStorageValue::Integer(count as i64),
            ],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_to_counts_empty() {
        assert!(rows_to_counts(vec![]).is_empty());
    }

    #[test]
    fn rows_to_counts_maps_correctly() {
        let rows = vec![
            Row {
                page: "posts/hello".into(),
                count: 42,
            },
            Row {
                page: "posts/world".into(),
                count: 100,
            },
        ];
        let counts = rows_to_counts(rows);
        assert_eq!(counts.len(), 2);
        assert_eq!(counts["posts/hello"], 42);
        assert_eq!(counts["posts/world"], 100);
    }

    #[test]
    fn rows_to_counts_clamps_negative_to_zero() {
        let rows = vec![
            Row {
                page: "posts/a".into(),
                count: -5,
            },
            Row {
                page: "posts/b".into(),
                count: 0,
            },
        ];
        let counts = rows_to_counts(rows);
        assert_eq!(counts["posts/a"], 0);
        assert_eq!(counts["posts/b"], 0);
    }

    #[test]
    fn rows_to_counts_last_row_wins_on_duplicate_page() {
        let rows = vec![
            Row {
                page: "posts/dup".into(),
                count: 10,
            },
            Row {
                page: "posts/dup".into(),
                count: 20,
            },
        ];
        let counts = rows_to_counts(rows);
        assert_eq!(counts["posts/dup"], 20);
    }
}
