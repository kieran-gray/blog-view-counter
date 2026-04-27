use serde::Deserialize;
use worker::{SqlStorage, SqlStorageValue};

#[derive(Deserialize)]
struct Row {
    count: i64,
}

fn rows_to_count(rows: Vec<Row>) -> u64 {
    rows.into_iter()
        .next()
        .map(|r| r.count.max(0) as u64)
        .unwrap_or(0)
}

pub fn init_schema(sql: &SqlStorage) -> worker::Result<()> {
    sql.exec(
        "CREATE TABLE IF NOT EXISTS views (
            id INTEGER PRIMARY KEY CHECK (id = 0),
            count INTEGER NOT NULL DEFAULT 0
        )",
        None,
    )?;
    Ok(())
}

pub fn load_count(sql: &SqlStorage) -> worker::Result<u64> {
    let rows: Vec<Row> = sql
        .exec("SELECT count FROM views WHERE id = 0", None)?
        .to_array()?;

    Ok(rows_to_count(rows))
}

pub fn flush_count(sql: &SqlStorage, count: u64) -> worker::Result<()> {
    sql.exec(
        "INSERT INTO views (id, count) VALUES (0, ?)
         ON CONFLICT(id) DO UPDATE SET count = excluded.count",
        vec![SqlStorageValue::Integer(count as i64)],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_to_count_empty() {
        assert_eq!(rows_to_count(vec![]), 0);
    }

    #[test]
    fn rows_to_count_maps_correctly() {
        let rows = vec![Row { count: 42 }];
        assert_eq!(rows_to_count(rows), 42);
    }

    #[test]
    fn rows_to_count_clamps_negative_to_zero() {
        let rows = vec![Row { count: -5 }];
        assert_eq!(rows_to_count(rows), 0);
    }

    #[test]
    fn rows_to_count_uses_first_row() {
        let rows = vec![Row { count: 10 }, Row { count: 20 }];

        assert_eq!(rows_to_count(rows), 10);
    }
}
