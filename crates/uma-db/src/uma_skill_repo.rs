use crate::types::{
    DbAptitudeLevel, DbSkillAcquisition, DbSkillCategory, DbSkillRarity, UmaRow, UmaSkillRow,
};
use sqlx::PgPool;

pub async fn get_skills_for_uma(
    pool: &PgPool,
    uma_id: i32,
) -> Result<Vec<UmaSkillRow>, sqlx::Error> {
    sqlx::query_as!(
        UmaSkillRow,
        r#"
        SELECT s.id, s.name, s.category as "category: DbSkillCategory",
            s.rarity as "rarity: DbSkillRarity",
            s.sp_cost, us.acquisition as "acquisition: DbSkillAcquisition",
            us.evolved_from
        FROM uma_skills us
        JOIN skills s ON s.id = us.skill_id
        WHERE us.uma_id = $1
        ORDER BY s.name
        "#,
        uma_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_uma_by_id(pool: &PgPool, id: i32) -> Result<Option<UmaRow>, sqlx::Error> {
    sqlx::query_as!(
        UmaRow,
        r#"
        SELECT id, name, subtitle,
            apt_turf as "apt_turf: DbAptitudeLevel",
            apt_dirt as "apt_dirt: DbAptitudeLevel",
            apt_short as "apt_short: DbAptitudeLevel",
            apt_mile as "apt_mile: DbAptitudeLevel",
            apt_medium as "apt_medium: DbAptitudeLevel",
            apt_long as "apt_long: DbAptitudeLevel",
            apt_front as "apt_front: DbAptitudeLevel",
            apt_pace as "apt_pace: DbAptitudeLevel",
            apt_late as "apt_late: DbAptitudeLevel",
            apt_end as "apt_end: DbAptitudeLevel",
            growth_speed, growth_stamina, growth_power, growth_guts, growth_wit
        FROM umas WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}
