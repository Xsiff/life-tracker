pub(super) const INIT_SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS activities (
        date     TEXT    NOT NULL,
        hour     INTEGER NOT NULL,
        category TEXT    NOT NULL,
        note     TEXT,
        PRIMARY KEY (date, hour)
    );

    CREATE TABLE IF NOT EXISTS day_notes (
        date     TEXT    NOT NULL,
        note     TEXT    NOT NULL,
        PRIMARY KEY (date)
    );
"#;
