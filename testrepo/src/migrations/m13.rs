//! Migration m13 - Database migration

pub fn up() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS m13_table (
        id INTEGER PRIMARY KEY,
        created_at TEXT NOT NULL,
        data TEXT
    );
    "#
}

pub fn down() -> &'static str {
    r#"
    DROP TABLE IF EXISTS m13_table;
    "#
}
