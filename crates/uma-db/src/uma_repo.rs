use crate::types::{DbAptitudeLevel, DbSkillAcquisition, DbUmaRarity};
use sqlx::PgPool;
use uma_core::models::uma::Uma;

pub async fn upsert_all_uma(pool: &PgPool, umas: &[Uma]) -> Result<(), sqlx::Error> {
    if umas.is_empty() {
        return Ok(());
    }

    let ids: Vec<i32> = umas.iter().map(|u| u.id.0 as i32).collect();
    let names: Vec<&str> = umas.iter().map(|u| u.name.as_str()).collect();
    let subtitles: Vec<&str> = umas.iter().map(|u| u.subtitle.as_str()).collect();
    let rarities: Vec<DbUmaRarity> = umas.iter().map(|u| DbUmaRarity::from(u.rarity)).collect();
    let stat_speeds: Vec<i32> = umas.iter().map(|u| u.base_stats.speed as i32).collect();
    let stat_staminas: Vec<i32> = umas.iter().map(|u| u.base_stats.stamina as i32).collect();
    let stat_powers: Vec<i32> = umas.iter().map(|u| u.base_stats.power as i32).collect();
    let stat_guts: Vec<i32> = umas.iter().map(|u| u.base_stats.guts as i32).collect();
    let stat_wits: Vec<i32> = umas.iter().map(|u| u.base_stats.wit as i32).collect();
    let growth_speeds: Vec<i32> = umas.iter().map(|u| u.growth_rates.speed as i32).collect();
    let growth_staminas: Vec<i32> = umas.iter().map(|u| u.growth_rates.stamina as i32).collect();
    let growth_powers: Vec<i32> = umas.iter().map(|u| u.growth_rates.power as i32).collect();
    let growth_guts: Vec<i32> = umas.iter().map(|u| u.growth_rates.guts as i32).collect();
    let growth_wits: Vec<i32> = umas.iter().map(|u| u.growth_rates.wit as i32).collect();
    let apt_turfs: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.surface.turf))
        .collect();
    let apt_dirts: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.surface.dirt))
        .collect();
    let apt_shorts: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.distance.short))
        .collect();
    let apt_miles: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.distance.mile))
        .collect();
    let apt_mediums: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.distance.medium))
        .collect();
    let apt_longs: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.distance.long))
        .collect();
    let apt_fronts: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.strategy.front))
        .collect();
    let apt_paces: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.strategy.pace))
        .collect();
    let apt_lates: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.strategy.late))
        .collect();
    let apt_ends: Vec<DbAptitudeLevel> = umas
        .iter()
        .map(|u| DbAptitudeLevel::from(u.aptitudes.strategy.end))
        .collect();

    sqlx::query!(
        r#"
        INSERT INTO umas (
            id, name, subtitle, rarity,
            stat_speed, stat_stamina, stat_power, stat_guts, stat_wit,
            growth_speed, growth_stamina, growth_power, growth_guts, growth_wit,
            apt_turf, apt_dirt,
            apt_short, apt_mile, apt_medium, apt_long,
            apt_front, apt_pace, apt_late, apt_end
        )
        SELECT * FROM UNNEST(
            $1::int[], $2::text[], $3::text[], $4::uma_rarity[],
            $5::int[], $6::int[], $7::int[], $8::int[], $9::int[],
            $10::int[], $11::int[], $12::int[], $13::int[], $14::int[],
            $15::aptitude_level[], $16::aptitude_level[],
            $17::aptitude_level[], $18::aptitude_level[], $19::aptitude_level[], $20::aptitude_level[],
            $21::aptitude_level[], $22::aptitude_level[], $23::aptitude_level[], $24::aptitude_level[]
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
        &ids,
        &names as &[&str],
        &subtitles as &[&str],
        &rarities as &[DbUmaRarity],
        &stat_speeds,
        &stat_staminas,
        &stat_powers,
        &stat_guts,
        &stat_wits,
        &growth_speeds,
        &growth_staminas,
        &growth_powers,
        &growth_guts,
        &growth_wits,
        &apt_turfs as &[DbAptitudeLevel],
        &apt_dirts as &[DbAptitudeLevel],
        &apt_shorts as &[DbAptitudeLevel],
        &apt_miles as &[DbAptitudeLevel],
        &apt_mediums as &[DbAptitudeLevel],
        &apt_longs as &[DbAptitudeLevel],
        &apt_fronts as &[DbAptitudeLevel],
        &apt_paces as &[DbAptitudeLevel],
        &apt_lates as &[DbAptitudeLevel],
        &apt_ends as &[DbAptitudeLevel],
    )
    .execute(pool)
    .await?;

    sqlx::query!("DELETE FROM uma_skills WHERE uma_id = ANY($1::int[])", &ids)
        .execute(pool)
        .await?;

    let mut uma_ids: Vec<i32> = Vec::new();
    let mut skill_ids: Vec<i32> = Vec::new();
    let mut acquisitions: Vec<DbSkillAcquisition> = Vec::new();
    let mut evolved_froms: Vec<Option<i32>> = Vec::new();

    for uma in umas {
        for skill in &uma.skill_list {
            uma_ids.push(uma.id.0 as i32);
            skill_ids.push(skill.id.0 as i32);
            acquisitions.push(DbSkillAcquisition::from(skill.acquisition));
            evolved_froms.push(skill.acquisition.evolved_from().map(|id| id.0 as i32));
        }
    }

    if !uma_ids.is_empty() {
        let result = sqlx::query!(
            r#"
INSERT INTO uma_skills (uma_id, skill_id, acquisition, evolved_from)
SELECT * FROM UNNEST($1::int[], $2::int[], $3::skill_acquisition[], $4::int[])
ON CONFLICT (uma_id, skill_id, acquisition) DO UPDATE SET
    evolved_from = EXCLUDED.evolved_from
            "#,
            &uma_ids,
            &skill_ids,
            &acquisitions as &[DbSkillAcquisition],
            &evolved_froms as &[Option<i32>],
        )
        .execute(pool)
        .await?;

        log::info!(
            "uma_skills insert: {} rows affected",
            result.rows_affected()
        );
    }

    log::info!(
        "Uma upsert complete: {} umas, {} skills",
        umas.len(),
        uma_ids.len(),
    );

    Ok(())
}
