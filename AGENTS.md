# AGENTS.md - Agentic Coding Guidelines

This file provides guidelines for agents operating in this repository.

## Build / Lint / Test Commands

### Development Environment
**Always enter the nix dev shell first:**
```bash
nix develop
```

### Build & Run
```bash
cargo build          # Build the project
cargo run            # Run the API server (requires PostgreSQL)
```

### Development
```bash
cargo check          # Type-check without building
cargo fmt            # Format code (run before committing)
cargo clippy         # Linter warnings
```

### Database Setup
- Requires PostgreSQL database configured via `.env` file
- Environment variables:

| Variable           | Purpose                    | Default        |
|--------------------|----------------------------|----------------|
| `POSTGRES_HOST`    | Database host              | —              |
| `POSTGRES_PORT`    | Database port              | —              |
| `POSTGRES_DB`      | Database name              | —              |
| `POSTGRES_USER`    | Database user              | —              |
| `POSTGRES_PASSWORD`| Database password          | —              |
| `PORT`             | HTTP server port           | `8080`         |
| `TESTING`          | Skip `.env` loading (e2e)  | —              |

- `.env.test` format (same keys, used for e2e tests):
```
POSTGRES_HOST=...
POSTGRES_PORT=5432
POSTGRES_USER="bill_keeper_testing"
POSTGRES_PASSWORD="..."
POSTGRES_DB=bill_keeper_testing
```

---

## Code Style Guidelines

### Import Conventions
- Use crate-relative paths for internal modules: `use crate::module::Item;`
- Group std library imports first, then external crates, then internal modules
- Prefer importing specific items rather than glob imports

Example:
```rust
use std::collections::HashMap;
use actix_web::{web, HttpResponse, Responder};
use crate::db::DbPool;
```

### Formatting
- Use `cargo fmt` for automatic formatting
- 4-space indentation
- Maximum line length: 100 characters
- Add trailing commas in struct literals and match arms

### Types
- Use explicit types in function signatures for clarity
- Prefer `i32` for database IDs and integer fields
- Use `String` for text fields, not `&str` in struct definitions
- Use `Option<T>` for nullable database fields
- Use `Decimal` from `rust_decimal` for monetary amounts

### Naming Conventions
- **Snake_case** for variables, functions, and modules
- **PascalCase** for structs, enums, and traits
- **SCREAMING_SNAKE_CASE** for constants
- Prefix unused variables with underscore: `_unused_var`

### Error Handling
- Return `HttpResponse` with appropriate status codes:
  - `HttpResponse::Ok().json(...)` for success
  - `HttpResponse::NotFound()` for not found
  - `HttpResponse::BadRequest()` for invalid input
  - `HttpResponse::InternalServerError()` for errors
- Use `log::error!` or `log::warn!` for logging errors
- Include meaningful error messages in JSON responses

Example handler pattern:
```rust
#[get("/endpoint")]
pub async fn handler(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let data_group = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    // ... handler logic
}
```

### Handler Organization
- One handler per file in `src/handlers/`
- Use query parameters for filtering (e.g., `?data_group=1`)
- Return JSON responses using `.json()`

### Database Queries
- Use parameterized queries to prevent SQL injection
- Cast numeric types in SQL when needed: `amount::text`
- Use `pool.get_client().await` to get database connection
- Use `crate::db::log_db_error()` for logging database errors

### Serialization
- Derive `Serialize, Deserialize, Debug, Clone` for model structs
- Use `serde_json::json!()` macro for creating JSON responses
- Handle null database fields with `.get::<_, Option<T>>()`

---

## Project Structure

```
src/
├── main.rs          # Entry point, server setup, CORS, logger
├── db/
│   └── mod.rs       # DbPool (deadpool-postgres), log_db_error()
├── handlers/        # HTTP route handlers (1 file per resource)
│   ├── mod.rs
│   ├── application_reports.rs
│   ├── bills.rs
│   ├── data_groups.rs
│   ├── ear.rs
│   ├── expenses.rs
│   ├── images.rs
│   ├── reports.rs
│   ├── summaries.rs
│   └── utild.rs
├── helpers/         # get_data_group_url(), sanitize_filename()
│   ├── mod.rs
│   └── helpers.rs
├── middleware/
│   ├── mod.rs
│   └── logging.rs   # RequestLogger — logs method/path/status/duration
├── models/          # Data structures split by role
│   ├── mod.rs
│   ├── entities.rs  # Bill, Expense, ApplicationReport, DataGroup, etc.
│   ├── requests.rs  # CreateExpenseRequest, BillUpdateRequest, etc.
│   └── responses.rs # Summary, EarTotals, Report, CsvImportResult, etc.
├── routes/
│   └── mod.rs       # Registers all handlers under /api scope
└── services/        # Business logic (calculations, image processing, PDF)
    ├── mod.rs       # calculate_summaries(), calculate_ear_totals(), etc.
    ├── image_processor.rs
    └── pdf_converter.rs
```

---

## CORS Configuration

In `main.rs`:
```rust
Cors::default()
    .allowed_origin("http://localhost:5173")
    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"])
    .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::ACCEPT])
    .supports_credentials()
    .max_age(3600);
```

- Only allows the Vite dev server origin. If the frontend origin or port changes, update this.

---

## API Endpoints Overview

All 26 endpoints under `/api/` scope:

| Resource             | Method   | Path                                        |
|----------------------|----------|---------------------------------------------|
| Bills                | GET      | `/api/bills`                                |
| Bills                | GET      | `/api/bills/{id}`                           |
| Bills                | PUT      | `/api/bills`                                |
| Bills                | POST     | `/api/bills/upload` (multipart)             |
| Bills                | DELETE   | `/api/bills/{id}`                           |
| Expenses             | GET      | `/api/expenses`                             |
| Expenses             | POST     | `/api/expenses`                             |
| Expenses             | POST     | `/api/expenses/bulk` (CSV import)           |
| Expenses             | PATCH    | `/api/expenses/{id}/bill`                   |
| Expenses             | PATCH    | `/api/expenses/{id}/type`                   |
| Expenses             | PATCH    | `/api/expenses/{id}/application`            |
| Expenses             | PATCH    | `/api/expenses/{id}/cash`                   |
| Expenses             | DELETE   | `/api/expenses/{id}`                        |
| Summaries            | GET      | `/api/summaries`                            |
| EAR                  | GET      | `/api/ear`                                  |
| Reports              | GET      | `/api/reports`                              |
| Images               | GET      | `/api/images/{filename}`                    |
| Data Groups          | GET      | `/api/data_groups`                          |
| Data Groups          | POST     | `/api/data_groups`                          |
| Data Groups          | DELETE   | `/api/data_groups/{id}` (cascading)         |
| Application Reports  | GET      | `/api/application_reports`                  |
| Application Reports  | POST     | `/api/application_reports`                  |
| Application Reports  | PATCH    | `/api/application_reports/{id}`             |
| Application Reports  | DELETE   | `/api/application_reports/{id}`             |
| Utility Data         | GET      | `/api/utild`                                |
| Utility Data         | PUT      | `/api/utild`                                |

All list endpoints require `?data_group=<id>` query parameter.
There is **no authentication, no pagination, no rate limiting**.

---

## Key Dependencies

- `actix-web` — HTTP framework
- `actix-cors` — CORS middleware
- `actix-multipart` — File upload handling (bills)
- `actix-files` — Static file serving (images)
- `deadpool-postgres` / `tokio-postgres` — Connection pool + async PostgreSQL
- `rust_decimal` — Monetary amounts
- `csv` — CSV import parsing
- `image` — Image compression/resizing
- `pdf2image` — PDF-to-JPG conversion for uploaded bills
- `simplelog` — Dual logging (terminal + file)

---

## Recommended Behavior Patterns

### Response Style
- Keep responses short (1-3 sentences)
- Avoid unnecessary preamble or postamble
- Answer directly without excessive explanation

### Code Output
- Minimum comments unless explicitly requested
- Follow existing code patterns in the codebase
- Run cargo fmt before committing

### Tool Usage
- Use read/edit tools first for code changes
- Use grep/glob for exploration
- Use Bash for commands only (no file operations)

---

## Common Operations

### Adding a New Handler
1. Create new file in `src/handlers/`
2. Define request/response models in `src/models/` (entities in `entities.rs`, request types in `requests.rs`, response types in `responses.rs`)
3. Register route in `src/routes/mod.rs`
4. Export module in `src/handlers/mod.rs`

### Running a Single Test
```bash
cargo test test_name_here
```

### Running E2E Tests
```bash
cargo test --test e2e -- --test-threads=1
```

**Important:** Tests MUST run sequentially due to shared database state (each test drops/recreates tables). Running with multiple threads will cause PostgreSQL catalog conflicts.

Each test auto-allocates a unique port (starting from 8090) to avoid AddressInUse between sequential tests.

When working on E2E tests, always use and or expand the TestError struct in `tests/src/error.rs`.
When working on the E2E tests, always check the models the backend is expecting to receive and the model the frontend is sending back. The root directory of the frontend can be found in `/home/peter/projects/bill_keeper/frontend/`.

E2E test suite covers **expenses** (8 tests: get, create, update_bill, update_type, update_application, update_cash, delete, bulk_import), **data_groups** (1 test: get), and **bills** (7 tests: get, get_by_id, update, delete, upload_jpg, upload_png, upload_pdf, upload_unsupported).

E2E tests require:
- PostgreSQL running with credentials in `.env.test` file (user: `bill_keeper_testing`, database: `bill_keeper_testing`)
- The test will:
  1. Connect to `bill_keeper_testing` database as `bill_keeper_testing` user
  2. DROP and CREATE tables from `schema.sql`
  3. INSERT test user with bcrypt hash, then seed data from `seed_data.sql`
  4. Spawn the API server binary on an auto-allocated port starting at 8090 (with `TESTING=true`)
  5. Login as `test`/`test` to get a JWT token, build an authenticated reqwest client
  6. Run all e2e tests sequentially
  7. Tear down: DROP tables and kill server

---

## Related Services

- **Frontend:** `/home/peter/projects/bill_keeper/frontend/` — React SPA, expects API on port 8080
- **Workers:** `/home/peter/projects/bill_keeper/workers/` — PDF/XLSX generation on port 8082
