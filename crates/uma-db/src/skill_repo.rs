use crate::types::{DbSkillCategory, DbSkillOperator, DbSkillRarity};
use sqlx::PgPool;
use uma_core::models::skill::{ConditionType, Skill};

pub async fn upsert_all_condition_types(
    pool: &PgPool,
    conditions: &[ConditionType],
) -> Result<(), sqlx::Error> {
    if conditions.is_empty() {
        return Ok(());
    }

    let cond_keys: Vec<&str> = conditions.iter().map(|c| c.cond_key.as_str()).collect();
    let descriptions: Vec<&str> = conditions.iter().map(|c| c.description.as_str()).collect();
    let examples: Vec<Option<&str>> = conditions.iter().map(|c| c.example.as_deref()).collect();

    sqlx::query!(
        r#"
        INSERT INTO skill_condition_types (cond_key, description, example)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
        ON CONFLICT (cond_key) DO UPDATE SET
            description = EXCLUDED.description,
            example = EXCLUDED.example
        "#,
        &cond_keys as &[&str],
        &descriptions as &[&str],
        &examples as &[Option<&str>],
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn upsert_all_skills(pool: &PgPool, skills: &[Skill]) -> Result<(), sqlx::Error> {
    if skills.is_empty() {
        return Ok(());
    }

    let ids: Vec<i32> = skills.iter().map(|s| s.id.0 as i32).collect();
    let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
    let descriptions: Vec<&str> = skills
        .iter()
        .map(|s| s.ingame_description.as_str())
        .collect();
    let categories: Vec<DbSkillCategory> = skills
        .iter()
        .map(|s| DbSkillCategory::from(s.category))
        .collect();
    let rarities: Vec<DbSkillRarity> = skills
        .iter()
        .map(|s| DbSkillRarity::from(s.rarity))
        .collect();
    let sp_costs: Vec<i32> = skills.iter().map(|s| s.sp_cost as i32).collect();
    let jp_only: Vec<bool> = skills.iter().map(|s| s.is_jp_only).collect();

    sqlx::query!(
        r#"
        INSERT INTO skills (id, name, ingame_description, category, rarity, sp_cost, is_jp_only)
        SELECT * FROM UNNEST($1::int[], $2::text[], $3::text[], $4::skill_category[], $5::skill_rarity[], $6::int[], $7::bool[])
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            ingame_description = EXCLUDED.ingame_description,
            category = EXCLUDED.category,
            rarity = EXCLUDED.rarity,
            sp_cost = EXCLUDED.sp_cost,
            is_jp_only = EXCLUDED.is_jp_only
        "#,
        &ids,
        &names as &[&str],
        &descriptions as &[&str],
        &categories as &[DbSkillCategory],
        &rarities as &[DbSkillRarity],
        &sp_costs,
        &jp_only,
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "DELETE FROM skill_triggers WHERE skill_id = ANY($1::int[])",
        &ids
    )
    .execute(pool)
    .await?;

    let trigger_skill_ids: Vec<i32> = skills
        .iter()
        .flat_map(|s| s.effects.iter().map(move |_| s.id.0 as i32))
        .collect();

    if trigger_skill_ids.is_empty() {
        return Ok(());
    }

    let trigger_ids = sqlx::query!(
        r#"
        INSERT INTO skill_triggers (skill_id)
        SELECT * FROM UNNEST($1::int[])
        RETURNING id
        "#,
        &trigger_skill_ids,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.id)
    .collect::<Vec<i32>>();

    let mut effect_trigger_ids: Vec<i32> = Vec::new();
    let mut effect_types: Vec<&str> = Vec::new();
    let mut effect_values: Vec<Option<i32>> = Vec::new();

    let mut cond_trigger_ids: Vec<i32> = Vec::new();
    let mut cond_keys: Vec<&str> = Vec::new();
    let mut cond_operators: Vec<DbSkillOperator> = Vec::new();
    let mut cond_vals: Vec<&str> = Vec::new();
    let mut cond_is_preconditions: Vec<bool> = Vec::new();
    let mut cond_is_ors: Vec<bool> = Vec::new();

    for (trigger, &trigger_id) in skills
        .iter()
        .flat_map(|s| s.effects.iter())
        .zip(trigger_ids.iter())
    {
        for effect in &trigger.effects {
            effect_trigger_ids.push(trigger_id);
            effect_types.push(effect.type_name());
            effect_values.push(effect.value());
        }

        for condition in &trigger.conditions {
            cond_trigger_ids.push(trigger_id);
            cond_keys.push(&condition.cond_key);
            cond_operators.push(DbSkillOperator::from(condition.operator));
            cond_vals.push(&condition.cond_val);
            cond_is_preconditions.push(false);
            cond_is_ors.push(condition.is_or);
        }

        for condition in &trigger.preconditions {
            cond_trigger_ids.push(trigger_id);
            cond_keys.push(&condition.cond_key);
            cond_operators.push(DbSkillOperator::from(condition.operator));
            cond_vals.push(&condition.cond_val);
            cond_is_preconditions.push(true);
            cond_is_ors.push(condition.is_or);
        }
    }

    if !effect_trigger_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO skill_trigger_effects (trigger_id, effect_type, effect_value)
            SELECT * FROM UNNEST($1::int[], $2::text[], $3::int[])
            "#,
            &effect_trigger_ids,
            &effect_types as &[&str],
            &effect_values as &[Option<i32>],
        )
        .execute(pool)
        .await?;
    }

    if !cond_trigger_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO skill_trigger_conditions
                (trigger_id, cond_key, operator, cond_val, is_precondition, is_or)
            SELECT * FROM UNNEST($1::int[], $2::text[], $3::skill_operator[], $4::text[], $5::bool[], $6::bool[])
            "#,
            &cond_trigger_ids,
            &cond_keys as &[&str],
            &cond_operators as &[DbSkillOperator],
            &cond_vals as &[&str],
            &cond_is_preconditions,
            &cond_is_ors,
        )
        .execute(pool)
        .await?;
    }

    log::info!(
        "Skill upsert complete: {} skills, {} triggers, {} effects, {} conditions",
        skills.len(),
        trigger_ids.len(),
        effect_trigger_ids.len(),
        cond_trigger_ids.len(),
    );

    Ok(())
}
