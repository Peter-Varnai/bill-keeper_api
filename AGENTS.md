# AGENTS.md - Agentic Coding Guidelines

This file provides guidelines for agents operating in this repository.

## Build / Lint / Test Commands

### Build & Run
```bash
cargo build          # Build the project
cargo run           # Run the API server (requires PostgreSQL)
```

### Development
```bash
cargo check        # Type-check without building
cargo fmt          # Format code (run before committing)
cargo clippy       # Linter warnings
```

### Database Setup
- Requires PostgreSQL database configured via `.env` file
- Environment variables: `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`

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
- Use PATCH for partial updates, PUT for full updates

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
├── main.rs          # Entry point, server setup
├── db/             # Database connection pool
├── handlers/        # HTTP route handlers
├── helpers/         # Utility functions
├── middleware/    # Custom middleware
├── models/         # Data models
├── routes/         # Route configuration
└��─ services/     # Business logic (PDF conversion)
```

---

## Recommended Behavior Patterns

<!-- Uncomment and customize the following as needed -->

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
2. Define request/response models in `src/models/`
3. Register route in `src/routes/mod.rs`
4. Export module in `src/handlers/mod.rs`


### Running a Single Test
```bash
cargo test test_name_here
```
