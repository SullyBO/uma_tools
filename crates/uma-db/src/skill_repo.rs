use crate::types::{DbSkillCategory, DbSkillOperator, DbSkillRarity};
use sqlx::PgPool;
use uma_core::models::skill::{ConditionType, Skill};

pub async fn upsert_all_condition_types(
    pool: &PgPool,
    conditions: &[ConditionType],
) -> Result<(), sqlx::Error> {
    for condition in conditions {
        sqlx::query!(
            r#"
            INSERT INTO skill_condition_types (cond_key, description, example)
            VALUES ($1, $2, $3)
            ON CONFLICT (cond_key) DO UPDATE SET
                description = EXCLUDED.description,
                example = EXCLUDED.example
            "#,
            condition.cond_key,
            condition.description,
            condition.example,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn upsert_all_skills(pool: &PgPool, skills: &[Skill]) -> Result<(), sqlx::Error> {
    let mut success = 0;
    let mut fail_reasons: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for skill in skills {
        match upsert_skill(pool, skill).await {
            Ok(_) => success += 1,
            Err(e) => {
                let reason = e.to_string();
                log::warn!(
                    "Failed to upsert skill {} (id: {}): {reason}",
                    skill.name,
                    skill.id.0
                );
                *fail_reasons.entry(reason).or_insert(0) += 1;
            }
        }
    }

    let failed = fail_reasons.values().sum::<usize>();

    log::info!(
        "Skill upsert complete: {} succeeded, {} failed out of {} total",
        success,
        failed,
        skills.len()
    );

    if !fail_reasons.is_empty() {
        log::info!("Failure breakdown:");
        let mut reasons: Vec<_> = fail_reasons.iter().collect();
        reasons.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (reason, count) in reasons {
            log::info!("  {count}x {reason}");
        }
    }

    Ok(())
}

async fn upsert_skill(pool: &PgPool, skill: &Skill) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO skills (id, name, ingame_description, category, rarity, sp_cost)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            ingame_description = EXCLUDED.ingame_description,
            category = EXCLUDED.category,
            rarity = EXCLUDED.rarity,
            sp_cost = EXCLUDED.sp_cost
        "#,
        skill.id.0 as i32,
        skill.name,
        skill.ingame_description,
        DbSkillCategory::from(skill.category) as DbSkillCategory,
        DbSkillRarity::from(skill.rarity) as DbSkillRarity,
        skill.sp_cost as i32,
    )
    .execute(pool)
    .await?;

    // Delete existing triggers so we can re-insert cleanly
    sqlx::query!(
        "DELETE FROM skill_triggers WHERE skill_id = $1",
        skill.id.0 as i32
    )
    .execute(pool)
    .await?;

    for trigger in &skill.effects {
        let trigger_id = sqlx::query!(
            r#"
            INSERT INTO skill_triggers (skill_id)
            VALUES ($1)
            RETURNING id
            "#,
            skill.id.0 as i32,
        )
        .fetch_one(pool)
        .await?
        .id;

        for effect_type in &trigger.effects {
            sqlx::query!(
                r#"
                INSERT INTO skill_trigger_effects (trigger_id, effect_type, effect_value)
                VALUES ($1, $2, $3)
                "#,
                trigger_id,
                effect_type.type_name(),
                effect_type.value(),
            )
            .execute(pool)
            .await?;
        }

        for condition in &trigger.conditions {
            sqlx::query!(
                r#"
                INSERT INTO skill_trigger_conditions
                    (trigger_id, cond_key, operator, cond_val, is_precondition, is_or)
                VALUES ($1, $2, $3, $4, false, $5)
                "#,
                trigger_id,
                condition.cond_key,
                DbSkillOperator::from(condition.operator) as DbSkillOperator,
                condition.cond_val,
                condition.is_or,
            )
            .execute(pool)
            .await?;
        }

        for condition in &trigger.preconditions {
            sqlx::query!(
                r#"
                INSERT INTO skill_trigger_conditions
                    (trigger_id, cond_key, operator, cond_val, is_precondition, is_or)
                VALUES ($1, $2, $3, $4, true, $5)
                "#,
                trigger_id,
                condition.cond_key,
                DbSkillOperator::from(condition.operator) as DbSkillOperator,
                condition.cond_val,
                condition.is_or,
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
