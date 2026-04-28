use crate::types::{DbAptitudeLevel, DbSkillAcquisition, DbUmaRarity};
use sqlx::PgPool;
use uma_core::models::uma::Uma;

pub async fn upsert_all_uma(pool: &PgPool, umas: &[Uma]) -> Result<(), sqlx::Error> {
    let mut success = 0;
    let mut fail_reasons: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for uma in umas {
        match upsert_uma_full(pool, uma).await {
            Ok(_) => success += 1,
            Err(e) => {
                let reason = e.to_string();
                log::warn!(
                    "Failed to upsert uma {} (id: {}): {reason}",
                    uma.name,
                    uma.id.0
                );
                *fail_reasons.entry(reason).or_insert(0) += 1;
            }
        }
    }

    let failed = fail_reasons.values().sum::<usize>();

    log::info!(
        "Uma upsert complete: {} succeeded, {} failed out of {} total",
        success,
        failed,
        umas.len()
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

async fn upsert_uma_full(pool: &PgPool, uma: &Uma) -> Result<(), sqlx::Error> {
    upsert_uma(pool, uma).await?;
    upsert_uma_skills(pool, uma).await?;
    log::info!(
        "Upserted uma {} {} (id: {}, skills: {})",
        uma.rarity,
        uma.name,
        uma.id.0,
        uma.skill_list.len()
    );
    Ok(())
}

async fn upsert_uma(pool: &PgPool, uma: &Uma) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO umas (
            id, name, subtitle, rarity,
            stat_speed, stat_stamina, stat_power, stat_guts, stat_wit,
            growth_speed, growth_stamina, growth_power, growth_guts, growth_wit,
            apt_turf, apt_dirt,
            apt_short, apt_mile, apt_medium, apt_long,
            apt_front, apt_pace, apt_late, apt_end
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8, $9,
            $10, $11, $12, $13, $14,
            $15, $16,
            $17, $18, $19, $20,
            $21, $22, $23, $24
        )
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            subtitle = EXCLUDED.subtitle,
            rarity = EXCLUDED.rarity,
            stat_speed = EXCLUDED.stat_speed,
            stat_stamina = EXCLUDED.stat_stamina,
            stat_power = EXCLUDED.stat_power,
            stat_guts = EXCLUDED.stat_guts,
            stat_wit = EXCLUDED.stat_wit,
            growth_speed = EXCLUDED.growth_speed,
            growth_stamina = EXCLUDED.growth_stamina,
            growth_power = EXCLUDED.growth_power,
            growth_guts = EXCLUDED.growth_guts,
            growth_wit = EXCLUDED.growth_wit,
            apt_turf = EXCLUDED.apt_turf,
            apt_dirt = EXCLUDED.apt_dirt,
            apt_short = EXCLUDED.apt_short,
            apt_mile = EXCLUDED.apt_mile,
            apt_medium = EXCLUDED.apt_medium,
            apt_long = EXCLUDED.apt_long,
            apt_front = EXCLUDED.apt_front,
            apt_pace = EXCLUDED.apt_pace,
            apt_late = EXCLUDED.apt_late,
            apt_end = EXCLUDED.apt_end
        "#,
        uma.id.0 as i32,
        uma.name,
        uma.subtitle,
        DbUmaRarity::from(uma.rarity) as DbUmaRarity,
        uma.base_stats.speed as i32,
        uma.base_stats.stamina as i32,
        uma.base_stats.power as i32,
        uma.base_stats.guts as i32,
        uma.base_stats.wit as i32,
        uma.growth_rates.speed as i32,
        uma.growth_rates.stamina as i32,
        uma.growth_rates.power as i32,
        uma.growth_rates.guts as i32,
        uma.growth_rates.wit as i32,
        DbAptitudeLevel::from(uma.aptitudes.surface.turf) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.surface.dirt) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.distance.short) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.distance.mile) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.distance.medium) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.distance.long) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.strategy.front) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.strategy.pace) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.strategy.late) as DbAptitudeLevel,
        DbAptitudeLevel::from(uma.aptitudes.strategy.end) as DbAptitudeLevel,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn upsert_uma_skills(pool: &PgPool, uma: &Uma) -> Result<(), sqlx::Error> {
    for skill in &uma.skill_list {
        let evolved_from = skill.acquisition.evolved_from().map(|id| id.0 as i32);

        sqlx::query!(
            r#"
            INSERT INTO uma_skills (uma_id, skill_id, acquisition, evolved_from)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (uma_id, skill_id) DO UPDATE SET
                acquisition = EXCLUDED.acquisition,
                evolved_from = EXCLUDED.evolved_from
            "#,
            uma.id.0 as i32,
            skill.id.0 as i32,
            DbSkillAcquisition::from(skill.acquisition) as DbSkillAcquisition,
            evolved_from,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
