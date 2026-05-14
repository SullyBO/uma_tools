use crate::types::{
    ConditionRow, DbSkillCategory, DbSkillOperator, DbSkillRarity, EffectRow, SkillDetail,
    SkillFilter, SkillRow, TriggerRow,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uma_core::models::skill::{ConditionType, Duration, Skill};

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

    let trigger_durations: Vec<Option<f32>> = skills
        .iter()
        .flat_map(|s| s.effects.iter().map(|e| match e.duration {
            Duration::Timed(v) => Some(v),
            Duration::Infinite => None,
        }))
        .collect();

    if trigger_skill_ids.is_empty() {
        return Ok(());
    }

    let trigger_ids = sqlx::query!(
        r#"
        INSERT INTO skill_triggers (skill_id, duration)
        SELECT * FROM UNNEST($1::int[], $2::real[])
        RETURNING id
        "#,
        &trigger_skill_ids,
        &trigger_durations as &[Option<f32>],
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.id)
    .collect::<Vec<i32>>();

    let mut effect_trigger_ids: Vec<i32> = Vec::new();
    let mut effect_types: Vec<&str> = Vec::new();
    let mut effect_values: Vec<Option<f32>> = Vec::new();

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
            SELECT * FROM UNNEST($1::int[], $2::text[], $3::real[])
            "#,
            &effect_trigger_ids,
            &effect_types as &[&str],
            &effect_values as &[Option<f32>],
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

pub async fn get_skills(pool: &PgPool, filter: SkillFilter) -> Result<Vec<SkillRow>, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT DISTINCT s.id, s.name, s.category, s.rarity, s.sp_cost, s.is_jp_only
         FROM skills s",
    );

    if filter.effect_type.is_some() {
        qb.push(" JOIN skill_triggers st ON st.skill_id = s.id");
        qb.push(" JOIN skill_trigger_effects ste ON ste.trigger_id = st.id");
    }

    qb.push(" WHERE 1=1");

    if let Some(v) = filter.category {
        qb.push(" AND s.category = ");
        qb.push_bind(v);
    }
    if let Some(v) = filter.rarity {
        qb.push(" AND s.rarity = ");
        qb.push_bind(v);
    }
    if let Some(v) = filter.is_jp_only {
        qb.push(" AND s.is_jp_only = ");
        qb.push_bind(v);
    }
    if let Some(v) = filter.effect_type {
        qb.push(" AND ste.effect_type ILIKE ");
        qb.push_bind(format!("%{}%", v));
    }

    qb.push(" ORDER BY s.name");

    qb.build_query_as::<SkillRow>().fetch_all(pool).await
}

pub async fn get_skill_by_id(pool: &PgPool, id: i32) -> Result<Option<SkillDetail>, sqlx::Error> {
    let skill = sqlx::query_as!(
        SkillRow,
        r#"
        SELECT id, name, category as "category: DbSkillCategory",
            rarity as "rarity: DbSkillRarity", sp_cost, is_jp_only
        FROM skills WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(skill) = skill else {
        return Ok(None);
    };

    let trigger_records = sqlx::query!(
        "SELECT id, duration FROM skill_triggers WHERE skill_id = $1",
        id
    )
    .fetch_all(pool)
    .await?;

    if trigger_records.is_empty() {
        return Ok(Some(SkillDetail {
            skill,
            triggers: vec![],
        }));
    }

    let trigger_ids: Vec<i32> = trigger_records.iter().map(|t| t.id).collect();

    let all_effects = sqlx::query_as!(
        EffectRow,
        "SELECT trigger_id, effect_type, effect_value FROM skill_trigger_effects WHERE trigger_id = ANY($1::int[])",
        &trigger_ids
    )
    .fetch_all(pool)
    .await?;

    let all_conditions = sqlx::query!(
        r#"
        SELECT trigger_id, cond_key, operator as "operator: DbSkillOperator",
            cond_val, is_precondition, is_or
        FROM skill_trigger_conditions WHERE trigger_id = ANY($1::int[])
        "#,
        &trigger_ids
    )
    .fetch_all(pool)
    .await?;

    let triggers = trigger_records
        .iter()
        .map(|t| {
            let effects = all_effects
                .iter()
                .filter(|e| e.trigger_id == t.id)
                .map(|e| EffectRow {
                    trigger_id: e.trigger_id,
                    effect_type: e.effect_type.clone(),
                    effect_value: e.effect_value,
                })
                .collect();

            let (preconditions, conditions) = all_conditions
                .iter()
                .filter(|c| c.trigger_id == t.id)
                .partition::<Vec<_>, _>(|c| c.is_precondition);

            let conditions: Vec<ConditionRow> = conditions
                .into_iter()
                .map(|c| ConditionRow {
                    cond_key: c.cond_key.clone(),
                    operator: c.operator,
                    cond_val: c.cond_val.clone(),
                    is_or: c.is_or,
                })
                .collect();

            let preconditions: Vec<ConditionRow> = preconditions
                .into_iter()
                .map(|c| ConditionRow {
                    cond_key: c.cond_key.clone(),
                    operator: c.operator,
                    cond_val: c.cond_val.clone(),
                    is_or: c.is_or,
                })
                .collect();

            TriggerRow {
                id: t.id,
                duration: t.duration,
                effects,
                conditions,
                preconditions,
            }
        })
        .collect();

    Ok(Some(SkillDetail { skill, triggers }))
}
