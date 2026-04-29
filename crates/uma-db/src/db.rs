use sqlx::{ConnectOptions, PgPool, postgres::PgConnectOptions};
use uma_core::models::{
    skill::{ConditionType, Skill},
    uma::Uma,
};
use std::str::FromStr;

pub struct Db {
    pub(crate) pool: PgPool,
}

impl Db {
pub async fn connect() -> Result<Self, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let opts = PgConnectOptions::from_str(&url)?
        .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_secs(5));
    let pool = PgPool::connect_with(opts).await?;
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
