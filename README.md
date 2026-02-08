# naorm_macro

A lightweight ORM derive macro built on top of **sqlx**. It generates SQL constants and helper query builders for your structs, so you can keep SQL centralized while still using `sqlx` types.

## Features

- `#[derive(NaormReg)]` to generate CRUD SQL strings.
- Works with `sqlx` drivers: `sqlite` (default), `mysql`, `postgres`.
- Field metadata with type, nullability, primary key, auto increment, and default value.

## Usage

Add the macro crate and `sqlx` to your project, then derive `NaormReg`:

````rust
// filepath: /Users/ablegao/code/naorm_macro/README.md
// ...existing code...
use naorm_macro::NaormReg;

#[derive(NaormReg, sqlx::FromRow)]
#[naorm_cfg(table_name = "book_note", driver = "sqlite")]
struct BookNote {
    #[naorm_cfg(primary_key, auto_increment)]
    id: i64,
    book_id: i64,
    content: String,
    note: Option<String>,
    color: Option<String>,
    created_at: i64,
}
// ...existing code...


# Generated API (from naorm_macro::naorm)
# The macro generates:

# Constants:

    PK, PK_AUTO_INCREMENT, NAORM_TABLE, NAORM_DB, NAORM_TABLE_TYPE
    SELECT_SQL, INSERT_SQL, UPDATE_SQL, DELETE_SQL
    NAORM_FIELDS

# Methods:

    insert_query(&mut self) -> sqlx::query::Query<...>
    update_query(&mut self) -> sqlx::query::Query<...>
    delete_query(&self) -> sqlx::query::Query<...>
    all_query() -> sqlx::query::QueryAs<...>
    filter_query(w: &str) -> sqlx::query::QueryAs<...>

# Attribute Configuration
    The macro accepts the naorm_cfg attribute:

        table_name = "..."
        db_name = "..."
        table_type = "..."
        driver = "sqlite" | "mysql" | "postgres"
        
    Field-level:

primary_key
auto_increment
default = "..."
Notes
Only named-field structs are supported.
Defaults are inferred if default is not specified.
SQL placeholders are generated as ?, matching the sqlx placeholder style.
Links
Macro entry: naorm_macro::naorm
Helper module: table_create::to_snake_case