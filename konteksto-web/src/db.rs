use sqlx::{SqlitePool, prelude::FromRow};

#[derive(FromRow)]
pub struct Attempt {
    pub word: String,
    pub score: u32,
}

pub struct SqliteClient(SqlitePool);

impl SqliteClient {
    pub fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }

    pub async fn register_guess(&self, word: &str, score: u32) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            r#"INSERT INTO guesses(word, score) VALUES($1, $2)"#,
            word,
            score
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn all_guesses(&self) -> sqlx::Result<Vec<Attempt>> {
        sqlx::query_as(r#"SELECT word, score FROM guesses"#)
            .fetch_all(&self.0)
            .await
    }

    pub async fn delete_all_guesses(&self)->sqlx::Result<()>{
        sqlx::query!(r#"DELETE FROM guesses"#)
            .execute(&self.0)
            .await?;
        Ok(())
    }
}
