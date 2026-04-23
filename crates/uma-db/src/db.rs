use sqlx::PgPool;
use uma_core::{
    ids::SkillId,
    models::skill::{Skill, SkillEffect},
    models::uma::Uma,
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

    pub async fn upsert_all_skills(
        &self,
        skills: &[(Skill, Vec<SkillEffect>)],
    ) -> Result<(), sqlx::Error> {
        crate::skill_repo::upsert_all_skills(&self.pool, skills).await
    }

    pub async fn upsert_all_skill_details(
        &self,
        pairs: &[(SkillId, Vec<SkillEffect>)],
    ) -> Result<(), sqlx::Error> {
        crate::skill_repo::upsert_all_skill_details(&self.pool, pairs).await
    }

    pub async fn upsert_uma_full(&self, uma: &Uma) -> Result<(), sqlx::Error> {
        crate::uma_repo::upsert_uma_full(&self.pool, uma).await
    }

    pub async fn get_all_skill_ids(&self) -> Result<Vec<SkillId>, sqlx::Error> {
        crate::skill_repo::get_all_skill_ids(&self.pool).await
    }
}
