use uma_core::models::support_card::Rarity;

const ALL_BREAKPOINTS: [u32; 11] = [1, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50];

pub fn lb_levels_for_rarity(rarity: &Rarity) -> [u32; 5] {
    match rarity {
        Rarity::R => [20, 25, 30, 35, 40],
        Rarity::SR => [25, 30, 35, 40, 45],
        Rarity::SSR => [30, 35, 40, 45, 50],
    }
}
/// Interpolates an effect value at a given level using linear interpolation
/// between the two surrounding matrix breakpoints, floored.
///
/// The effects matrix encodes values at fixed level breakpoints (1, 5, 10, 15, 20,
/// 25, 30, 35, 40, 45, 50). A value of `-1` means "no change at this breakpoint".
/// Between two defined breakpoints, values increase linearly and are floored to
/// the nearest integer.
///
/// # Edge cases
/// - Returns `0` if no breakpoints are defined, or if `target_level` is below
///   the first defined breakpoint.
/// - Clamps to the final breakpoint value if `target_level` exceeds it.
///
/// # Example
/// Given breakpoints at lv1=5, lv20=10, lv25=10, lv40=15:
/// - At lv20: returns 10 (exact match)
/// - At lv30: returns 13 (floor(10 + 5 * 5/15))
/// - At lv50: returns 15 (clamped to final value)
pub fn interpolate_effect(matrix_values: &[i64], target_level: u32) -> i64 {
    let breakpoints: Vec<(u32, i64)> = matrix_values
        .iter()
        .enumerate()
        .filter(|(_, v)| **v != -1)
        .map(|(i, v)| (ALL_BREAKPOINTS[i], *v))
        .collect();

    if breakpoints.is_empty() {
        return 0;
    }

    if let Some(&(_, v)) = breakpoints.iter().find(|&&(lvl, _)| lvl == target_level) {
        return v;
    }

    if target_level < breakpoints[0].0 {
        return 0;
    }

    if target_level > breakpoints[breakpoints.len() - 1].0 {
        return breakpoints[breakpoints.len() - 1].1;
    }

    let upper_idx = breakpoints
        .iter()
        .position(|&(lvl, _)| lvl > target_level)
        .unwrap();
    let (lo_lvl, lo_val) = breakpoints[upper_idx - 1];
    let (hi_lvl, hi_val) = breakpoints[upper_idx];

    let delta = hi_val - lo_val;
    let segment_len = (hi_lvl - lo_lvl) as i64;
    let steps_from_lo = (target_level - lo_lvl) as i64;

    lo_val + (delta * steps_from_lo) / segment_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_effect_exact_breakpoint() {
        let values = vec![5, -1, -1, 10, 10, -1, -1, 15, -1, -1, -1];
        assert_eq!(interpolate_effect(&values, 20), 10);
    }

    #[test]
    fn interpolate_effect_between_breakpoints() {
        let values = vec![5, -1, -1, 10, 10, -1, -1, 15, -1, -1, -1];
        assert_eq!(interpolate_effect(&values, 30), 13);
    }

    #[test]
    fn interpolate_effect_below_first_breakpoint() {
        let values = vec![-1, -1, -1, -1, 10, -1, -1, -1, -1, -1, -1];
        assert_eq!(interpolate_effect(&values, 1), 0);
    }
}
