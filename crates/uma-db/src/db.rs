use sqlx::PgPool;

pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn connect(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(db_url).await?;
        Ok(Self { pool })
    }
}
