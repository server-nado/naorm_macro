use std::clone;
use std::fmt::format;

use naorm_macro::NaormReg;
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::any::AnyArguments;
use sqlx::{query::Query, Database, Error, SqlitePool};

pub struct BookNote {
    pub id: i64,
    pub book_id: i64,
    pub content: String,
    pub note: Option<String>,
    pub color: Option<String>,
    pub created_at: i64,
}
impl BookNote {
    pub const PK: &'static str = "id";
    pub const PK_AUTO_INCREMENT: bool = true;
    pub const NAORM_TABLE: &'static str = "book_note";
    pub const NAORM_DB: &'static str = "";
    pub const NAORM_TABLE_TYPE: &'static str = "";
    pub const SELECT_SQL: &'static str =
        "SELECT id, book_id, content, note, color, created_at FROM book_note";
    pub const INSERT_SQL: &'static str =
        "INSERT INTO book_note (book_id, content, note, color, created_at) VALUES (?, ?, ?, ?, ?)";
    pub const UPDATE_SQL: &'static str = "UPDATE book_note SET book_id = ?, content = ?, note = ?, color = ?, created_at = ? WHERE id = ?";
    pub const DELETE_SQL: &'static str = "DELETE FROM book_note WHERE id = ?";
    pub const NAORM_FIELDS: &'static [(
        &'static str,
        &'static str,
        bool,
        bool,
        bool,
        &'static str,
    )] = &[
        ("id", "i64", false, true, true, "0"),
        ("book_id", "i64", false, false, false, "0"),
        ("content", "String", false, false, false, ""),
        ("note", "String", true, false, false, ""),
        ("color", "String", true, false, false, ""),
        ("created_at", "i64", false, false, false, "0"),
    ];
    pub async fn create_table(pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS book_notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            note TEXT,
            color TEXT,
            created_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS book_notes_book_id_index on book_notes (book_id);
        CREATE VIRTUAL TABLE IF NOT EXISTS book_notes_fts USING fts5(
            content,
            note,
            content='book_notes',
            content_rowid='id'
        );
        CREATE TRIGGER IF NOT EXISTS book_notes_ai AFTER INSERT ON book_notes BEGIN
            INSERT INTO book_notes_fts (rowid, content, note) VALUES (new.id, new.content, new.note);
        END;
        ",
        )
        .execute(pool)
        .await?;
        Ok(())
    }
    pub fn insert_query<'q>(
        &'q mut self,
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query(Self::INSERT_SQL)
            .bind(&self.book_id)
            .bind(self.content.as_str())
            .bind(self.note.as_deref())
            .bind(self.color.as_deref())
            .bind(&self.created_at)
    }
    pub fn update_query<'q>(
        &'q mut self,
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query(Self::UPDATE_SQL)
            .bind(&self.book_id)
            .bind(self.content.as_str())
            .bind(self.note.as_deref())
            .bind(self.color.as_deref())
            .bind(&self.created_at)
            .bind(&self.id)
    }
    pub fn delete_query<'q>(
        &'q self,
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query(Self::DELETE_SQL).bind(&self.id)
    }
    pub fn all_query(
    ) -> sqlx::query::QueryAs<'static, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'static>>
    where
        Self: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        sqlx::query_as::<sqlx::Sqlite, Self>(Self::SELECT_SQL)
    }
    pub fn filter_query<'q>(
        w: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>>
    where
        Self: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        sqlx::query_as::<sqlx::Sqlite, Self>(w)
    }
}
#[derive(NaormReg, sqlx::FromRow, Serialize, Deserialize, Debug)]
struct MyStruct {
    id: i32,
    name: String,
    active: bool,
}
#[tokio::test]
async fn test_mystruct_naorm() {
    use sqlx;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

    BookNote::create_table(&pool).await.unwrap();

    let mut b = BookNote {
        id: 1,
        book_id: 2,
        content: "Sample content".to_string(),
        note: Some("This is a note".to_string()),
        color: Some("red".to_string()),
        created_at: 1625159073,
    };
    b.insert_query().execute(&pool).await.unwrap();
}
