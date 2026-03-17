use rusqlite::Connection;
use std::sync::Mutex;

pub struct DbPool {
    pub bills_conn: Mutex<Connection>,
    pub expenses_conn: Mutex<Connection>,
    pub data_groups_conn: Mutex<Connection>,
}

impl DbPool {
    pub fn new() -> Self {
        let bills_conn = Connection::open("./db/bills.db").expect("ERROR CONNECTING TO BILLS DB");
        let expenses_conn =
            Connection::open("./db/expenses.db").expect("ERROR CONNECTING TO EXPENSES DB");
        let data_groups_conn =
            Connection::open("./db/data_groups.db").expect("ERROR CONNECTING TO DATA_GROUPS DB");

        // Initialize data_groups table
        Self::initialize_data_groups_table(&data_groups_conn);

        // Initialize application_reports table
        Self::initialize_application_reports_table(&data_groups_conn);

        DbPool {
            bills_conn: Mutex::new(bills_conn),
            expenses_conn: Mutex::new(expenses_conn),
            data_groups_conn: Mutex::new(data_groups_conn),
        }
    }

    fn initialize_data_groups_table(conn: &Connection) {
        // Check if table exists and has correct schema
        let table_info: Result<Vec<(String, String)>, _> = conn
            .prepare("PRAGMA table_info(data_groups)")
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
                })?;
                rows.collect()
            });

        match table_info {
            Ok(columns) => {
                // Check if we have all required columns
                let column_names: Vec<String> =
                    columns.iter().map(|(name, _)| name.clone()).collect();
                let required = vec![
                    "id",
                    "name",
                    "type",
                    "expenses_table_name",
                    "bills_table_name",
                    "bills_storage_path",
                ];

                let has_all = required
                    .iter()
                    .all(|r| column_names.contains(&r.to_string()));

                if !has_all {
                    // Drop existing table and recreate
                    conn.execute("DROP TABLE IF EXISTS data_groups", [])
                        .expect("Failed to drop old data_groups table");
                    Self::create_table_and_insert_initial(conn);
                } else {
                    // Check if we need to add initial data
                    Self::insert_initial_if_empty(conn);
                }
            }
            Err(_) => {
                // Table doesn't exist, create it
                Self::create_table_and_insert_initial(conn);
            }
        }
    }

    fn create_table_and_insert_initial(conn: &Connection) {
        conn.execute(
            "CREATE TABLE data_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                type TEXT NOT NULL CHECK (type IN ('project', 'organization')),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                expenses_table_name TEXT NOT NULL,
                bills_table_name TEXT NOT NULL,
                bills_storage_path TEXT NOT NULL DEFAULT 'pdf_imgs/'
            )",
            [],
        )
        .expect("Failed to create data_groups table");

        Self::insert_initial_if_empty(conn);
    }

    fn insert_initial_if_empty(conn: &Connection) {
        // Check if we need to add existing 2024 data as initial group
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM data_groups", [], |row| row.get(0))
            .unwrap_or(0);

        if count == 0 {
            // Add existing 2024 data as first group
            conn.execute(
                "INSERT INTO data_groups (name, type, expenses_table_name, bills_table_name, bills_storage_path) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                ["2024", "project", "ear24_main", "bills24", "pdf_imgs/"],
            )
            .expect("Failed to insert initial data group");
        }
    }

    /// Helper function to lookup table names for a given data group ID
    /// Returns (expenses_table_name, bills_table_name) or error if group not found
    pub fn get_table_names(
        data_groups_conn: &Connection,
        group_id: i32,
    ) -> Result<(String, String), rusqlite::Error> {
        let result: (String, String) = data_groups_conn.query_row(
            "SELECT expenses_table_name, bills_table_name 
             FROM data_groups 
             WHERE id = ?1",
            [group_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(result)
    }

    /// Helper function to lookup bills storage path for a given data group ID
    pub fn get_bills_storage_path(
        data_groups_conn: &Connection,
        group_id: i32,
    ) -> Result<String, rusqlite::Error> {
        let path: String = data_groups_conn.query_row(
            "SELECT bills_storage_path 
             FROM data_groups 
             WHERE id = ?1",
            [group_id],
            |row| row.get(0),
        )?;
        Ok(path)
    }

    /// Initialize application_reports table
    fn initialize_application_reports_table(conn: &Connection) {
        // Check if table exists
        let table_exists: Result<i64, _> = conn
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='application_reports'", [], |row| row.get(0))
            .map(|count: i64| count);

        match table_exists {
            Ok(0) | Err(_) => {
                // Table doesn't exist, create it
                conn.execute(
                    "CREATE TABLE application_reports (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data_group_id INTEGER NOT NULL,
                        name TEXT NOT NULL,
                        amount REAL DEFAULT 0,
                        date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        submission_deadline TEXT,
                        FOREIGN KEY (data_group_id) REFERENCES data_groups(id)
                    )",
                    [],
                )
                .expect("Failed to create application_reports table");

                // Seed data for 2024 group
                Self::seed_application_reports_for_2024(conn);
            }
            Ok(_) => {
                // Table exists, check if we need to seed data for 2024
                Self::seed_application_reports_for_2024(conn);
            }
        }
    }

    /// Seed application reports for 2024 group if they don't exist
    fn seed_application_reports_for_2024(conn: &Connection) {
        // Find the 2024 data group
        let group_id: Option<i32> = conn
            .query_row(
                "SELECT id FROM data_groups WHERE name = '2024'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(gid) = group_id {
            // Check if application_reports already exist for this group
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM application_reports WHERE data_group_id = ?1",
                    [gid],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if count == 0 {
                // Seed the three default application reports
                conn.execute(
                    "INSERT INTO application_reports (data_group_id, name, amount) VALUES (?1, 'BMKOS', 27000)",
                    [gid],
                ).expect("Failed to insert BMKOS application report");

                conn.execute(
                    "INSERT INTO application_reports (data_group_id, name, amount) VALUES (?1, 'MA7', 8000)",
                    [gid],
                ).expect("Failed to insert MA7 application report");

                conn.execute(
                    "INSERT INTO application_reports (data_group_id, name, amount) VALUES (?1, 'Bezirk', 8000)",
                    [gid],
                ).expect("Failed to insert Bezirk application report");
            }
        }
    }
}
