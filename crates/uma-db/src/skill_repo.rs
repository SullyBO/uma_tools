use crate::types::{DbSkillCategory, DbSkillOperator, DbSkillRarity};
use sqlx::PgPool;
use uma_core::models::skill::{Skill, SkillEffect};

pub async fn upsert_all_skills(
    pool: &PgPool,
    skills: &[(Skill, Vec<SkillEffect>)],
) -> Result<(), sqlx::Error> {
    let mut cond_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (_, effects) in skills {
        for effect in effects {
            for condition in &effect.conditions {
                cond_keys.insert(condition.cond_key.clone());
            }
        }
    }

    let mut cond_key_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for cond_key in cond_keys {
        let id = sqlx::query!(
            r#"
            INSERT INTO skill_condition_types (cond_key)
            VALUES ($1)
            ON CONFLICT (cond_key) DO UPDATE SET cond_key = skill_condition_types.cond_key
            RETURNING id
            "#,
            cond_key,
        )
        .fetch_one(pool)
        .await?
        .id;

        cond_key_map.insert(cond_key, id);
    }

    let mut success = 0;
    let mut failed = 0;

    for (skill, effects) in skills {
        match upsert_skill_full(pool, skill, effects, &cond_key_map).await {
            Ok(_) => success += 1,
            Err(e) => {
                log::warn!(
                    "Failed to upsert skill {} (id: {}): {e}",
                    skill.name,
                    skill.id.0
                );
                failed += 1;
            }
        }
    }

    log::info!(
        "Skill upsert complete: {} succeeded, {} failed out of {} total",
        success,
        failed,
        skills.len()
    );

    Ok(())
}

pub async fn upsert_skill_full(
    pool: &PgPool,
    skill: &Skill,
    effects: &[SkillEffect],
    cond_key_map: &std::collections::HashMap<String, i32>,
) -> Result<(), sqlx::Error> {
    upsert_skill(pool, skill).await?;

    for effect in effects {
        let effect_id = sqlx::query!(
            r#"
            INSERT INTO skill_effects (skill_id)
            VALUES ($1)
            RETURNING id
            "#,
            skill.id.0 as i32,
        )
        .fetch_one(pool)
        .await?
        .id;

        for stat in &effect.stats {
            sqlx::query!(
                r#"
                INSERT INTO skill_effect_stats (effect_id, stat_key, stat_val)
                VALUES ($1, $2, $3)
                "#,
                effect_id,
                stat.stat_key,
                stat.stat_val,
            )
            .execute(pool)
            .await?;
        }

        for condition in &effect.conditions {
            let condition_type_id = cond_key_map[&condition.cond_key];
            sqlx::query!(
                r#"
                INSERT INTO skill_conditions (effect_id, condition_type_id, is_precondition, operator, cond_val)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                effect_id,
                condition_type_id,
                condition.is_precondition,
                DbSkillOperator::from(condition.operator) as DbSkillOperator,
                condition.cond_val,
            )
            .execute(pool)
            .await?;
        }
    }

    log::info!(
        "Upserted skill {} (id: {}, effects: {})",
        skill.name,
        skill.id.0,
        effects.len()
    );
    Ok(())
}

pub async fn upsert_skill(pool: &PgPool, skill: &Skill) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO skills (id, name, description, category, rarity, sp_cost, eval_points)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            category = EXCLUDED.category,
            rarity = EXCLUDED.rarity,
            sp_cost = EXCLUDED.sp_cost,
            eval_points = EXCLUDED.eval_points
        "#,
        skill.id.0 as i32,
        skill.name,
        skill.ingame_description,
        DbSkillCategory::from(skill.category) as DbSkillCategory,
        DbSkillRarity::from(skill.rarity) as DbSkillRarity,
        skill.sp_cost as i32,
        skill.eval_points as i32,
    )
    .execute(pool)
    .await?;

    log::info!("Upserted skill {} (id: {})", skill.name, skill.id.0);
    Ok(())
}
