use anyhow::Result;
use rusqlite::Connection;

use super::pitch_seq::PitchTransitionSet;

pub struct PitcherProfile {
    pub pitcher_id: i64,
    pub season: i32,
    pub overall_entropy: f64,
    pub total_pitches: i64,
    pub by_count: Vec<CountEntropy>,
}

pub struct CountEntropy {
    pub count_state: String,
    pub entropy: f64,
    pub pitches: i64,
}

pub fn shannon_entropy(probs: &[f64]) -> f64 {
    -probs
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| p * p.ln())
        .sum::<f64>()
}

pub fn compute_pitcher_profile(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
    transitions: &PitchTransitionSet,
) -> Result<PitcherProfile> {
    // Overall entropy: flatten the matrix into a pitch distribution
    let overall_entropy = matrix_entropy(&transitions.overall.matrix, &transitions.overall.occurrences);
    let total_pitches: i64 = transitions
        .overall
        .occurrences
        .iter()
        .flat_map(|row| row.iter())
        .sum();

    let mut by_count: Vec<CountEntropy> = Vec::new();
    for (cs, mat) in &transitions.by_count {
        let e = matrix_entropy(&mat.matrix, &mat.occurrences);
        let pitches: i64 = mat.occurrences.iter().flat_map(|row| row.iter()).sum();
        by_count.push(CountEntropy {
            count_state: cs.clone(),
            entropy: e,
            pitches,
        });
    }
    by_count.sort_by(|a, b| a.count_state.cmp(&b.count_state));

    // Store in DB
    store_profile(conn, pitcher_id, season, overall_entropy, total_pitches, &by_count)?;

    Ok(PitcherProfile {
        pitcher_id,
        season,
        overall_entropy,
        total_pitches,
        by_count,
    })
}

/// Average row entropy weighted by row occurrence count
fn matrix_entropy(matrix: &[Vec<f64>], occurrences: &[Vec<i64>]) -> f64 {
    let mut total_weight = 0i64;
    let mut weighted_entropy = 0.0;

    for (i, row) in matrix.iter().enumerate() {
        let row_count: i64 = occurrences[i].iter().sum();
        if row_count == 0 {
            continue;
        }
        let e = shannon_entropy(row);
        weighted_entropy += row_count as f64 * e;
        total_weight += row_count;
    }

    if total_weight > 0 {
        weighted_entropy / total_weight as f64
    } else {
        0.0
    }
}

fn store_profile(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
    overall_entropy: f64,
    total_pitches: i64,
    by_count: &[CountEntropy],
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM pitcher_profiles WHERE pitcher_id = ?1 AND season = ?2",
        rusqlite::params![pitcher_id, season],
    )?;

    // Overall
    tx.execute(
        "INSERT INTO pitcher_profiles (pitcher_id, season, count_state, entropy, total_pitches)
         VALUES (?1, ?2, NULL, ?3, ?4)",
        rusqlite::params![pitcher_id, season, overall_entropy, total_pitches],
    )?;

    // Per count
    for ce in by_count {
        tx.execute(
            "INSERT INTO pitcher_profiles (pitcher_id, season, count_state, entropy, total_pitches)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![pitcher_id, season, ce.count_state, ce.entropy, ce.pitches],
        )?;
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_shannon_entropy_uniform_2() {
        let result = shannon_entropy(&[0.5, 0.5]);
        assert_abs_diff_eq!(result, std::f64::consts::LN_2, epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_entropy_uniform_4() {
        let result = shannon_entropy(&[0.25, 0.25, 0.25, 0.25]);
        assert_abs_diff_eq!(result, 4_f64.ln(), epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_entropy_certain() {
        let result = shannon_entropy(&[1.0, 0.0, 0.0]);
        assert_abs_diff_eq!(result, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_entropy_empty() {
        let result = shannon_entropy(&[]);
        assert_abs_diff_eq!(result, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_entropy_skewed() {
        let result = shannon_entropy(&[0.9, 0.1]);
        let expected = -(0.9_f64 * 0.9_f64.ln() + 0.1_f64 * 0.1_f64.ln());
        assert_abs_diff_eq!(result, expected, epsilon = 1e-10);
    }

    #[test]
    fn test_shannon_entropy_zeros_filtered() {
        let result = shannon_entropy(&[0.5, 0.0, 0.0, 0.5]);
        assert_abs_diff_eq!(result, std::f64::consts::LN_2, epsilon = 1e-10);
    }

    #[test]
    fn test_matrix_entropy_uniform_rows() {
        // Both rows are [0.5, 0.5] — each has entropy ln(2)
        // Row 0 weight: 10+10 = 20, Row 1 weight: 20+20 = 40
        // Weighted average: (20*ln(2) + 40*ln(2)) / 60 = ln(2)
        let matrix = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let occurrences = vec![vec![10i64, 10], vec![20, 20]];
        let result = matrix_entropy(&matrix, &occurrences);
        assert_abs_diff_eq!(result, std::f64::consts::LN_2, epsilon = 1e-10);
    }

    #[test]
    fn test_matrix_entropy_mixed_rows() {
        // Row 0: [1.0, 0.0] — entropy 0, weight 30
        // Row 1: [0.5, 0.5] — entropy ln(2), weight 20
        // Weighted average: (30*0 + 20*ln(2)) / 50 = 20*ln(2)/50
        let matrix = vec![vec![1.0, 0.0], vec![0.5, 0.5]];
        let occurrences = vec![vec![30i64, 0], vec![10, 10]];
        let result = matrix_entropy(&matrix, &occurrences);
        let expected = 20.0 * std::f64::consts::LN_2 / 50.0;
        assert_abs_diff_eq!(result, expected, epsilon = 1e-10);
    }
}
