use sqlx::PgPool;
use uma_core::models::{
    skill::{ConditionType, Skill},
    uma::Uma,
};

pub struct Db {
    pub(crate) pool: PgPool,
}

impl Db {
    pub async fn connect() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await?;
        Ok(Self { pool })
    }

    pub async fn upsert_all_skills(&self, skill: &[Skill]) -> Result<(), sqlx::Error> {
        crate::skill_repo::upsert_all_skills(&self.pool, skill).await
    }

    pub async fn upsert_all_uma(&self, uma: &[Uma]) -> Result<(), sqlx::Error> {
        crate::uma_repo::upsert_all_uma(&self.pool, uma).await
    }

    pub async fn upsert_all_condition_types(
        &self,
        conditions: &[ConditionType],
    ) -> Result<(), sqlx::Error> {
        crate::skill_repo::upsert_all_condition_types(&self.pool, conditions).await
    }
}
