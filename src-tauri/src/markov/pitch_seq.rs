use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;

pub struct PitchMatrix {
    pub types: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub occurrences: Vec<Vec<i64>>,
}

pub struct PitchTransitionSet {
    pub pitcher_id: i64,
    pub season: i32,
    pub pitch_types: Vec<String>,
    pub overall: PitchMatrix,
    pub by_count: HashMap<String, PitchMatrix>,
}

pub fn compute_pitch_transitions(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
) -> Result<PitchTransitionSet> {
    // Get consecutive pitch pairs within the same at-bat for this pitcher
    let mut stmt = conn.prepare(
        "SELECT p1.pitch_type, p2.pitch_type,
                printf('%d-%d', p2.count_balls, p2.count_strikes)
         FROM pitches p1
         JOIN pitches p2 ON p1.play_id = p2.play_id AND p2.pitch_number = p1.pitch_number + 1
         JOIN plays pl ON pl.id = p1.play_id
         JOIN games g ON g.game_pk = pl.game_pk
         WHERE pl.pitcher_id = ?1
           AND strftime('%Y', g.game_date) = ?2
           AND p1.pitch_type != ''
           AND p2.pitch_type != ''",
    )?;

    // Collect all transitions grouped by count state
    let mut overall_counts: HashMap<(String, String), i64> = HashMap::new();
    let mut by_count_counts: HashMap<String, HashMap<(String, String), i64>> = HashMap::new();
    let mut all_types: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    let mut rows = stmt.query(rusqlite::params![pitcher_id, season.to_string()])?;
    while let Some(row) = rows.next()? {
        let from: String = row.get(0)?;
        let to: String = row.get(1)?;
        let count_state: String = row.get(2)?;

        all_types.insert(from.clone());
        all_types.insert(to.clone());

        *overall_counts.entry((from.clone(), to.clone())).or_default() += 1;
        *by_count_counts
            .entry(count_state)
            .or_default()
            .entry((from, to))
            .or_default() += 1;
    }

    let pitch_types: Vec<String> = all_types.into_iter().collect();

    let overall = build_matrix(&pitch_types, &overall_counts);

    let mut by_count = HashMap::new();
    for (cs, transitions) in &by_count_counts {
        by_count.insert(cs.clone(), build_matrix(&pitch_types, transitions));
    }

    // Store in DB
    store_pitch_transitions(conn, pitcher_id, season, &pitch_types, &overall, &by_count)?;

    Ok(PitchTransitionSet {
        pitcher_id,
        season,
        pitch_types,
        overall,
        by_count,
    })
}

fn build_matrix(
    types: &[String],
    counts: &HashMap<(String, String), i64>,
) -> PitchMatrix {
    let n = types.len();
    let type_idx: HashMap<&str, usize> = types.iter().enumerate().map(|(i, t)| (t.as_str(), i)).collect();

    let mut occ = vec![vec![0i64; n]; n];
    let mut matrix = vec![vec![0.0f64; n]; n];

    for ((from, to), &count) in counts {
        if let (Some(&fi), Some(&ti)) = (type_idx.get(from.as_str()), type_idx.get(to.as_str())) {
            occ[fi][ti] = count;
        }
    }

    // Row-normalize
    for i in 0..n {
        let row_sum: i64 = occ[i].iter().sum();
        if row_sum > 0 {
            for j in 0..n {
                matrix[i][j] = occ[i][j] as f64 / row_sum as f64;
            }
        }
    }

    PitchMatrix {
        types: types.to_vec(),
        matrix,
        occurrences: occ,
    }
}

fn store_pitch_transitions(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
    types: &[String],
    overall: &PitchMatrix,
    by_count: &HashMap<String, PitchMatrix>,
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM pitch_transitions WHERE pitcher_id = ?1 AND season = ?2",
        rusqlite::params![pitcher_id, season],
    )?;

    let n = types.len();

    // Overall (count_state = NULL)
    for i in 0..n {
        for j in 0..n {
            if overall.occurrences[i][j] == 0 {
                continue;
            }
            tx.execute(
                "INSERT INTO pitch_transitions
                 (pitcher_id, season, count_state, pitch_from, pitch_to, occurrences, probability)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    pitcher_id,
                    season,
                    types[i],
                    types[j],
                    overall.occurrences[i][j],
                    overall.matrix[i][j],
                ],
            )?;
        }
    }

    // Per count state
    for (cs, mat) in by_count {
        for i in 0..n {
            for j in 0..n {
                if mat.occurrences[i][j] == 0 {
                    continue;
                }
                tx.execute(
                    "INSERT INTO pitch_transitions
                     (pitcher_id, season, count_state, pitch_from, pitch_to, occurrences, probability)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        pitcher_id,
                        season,
                        cs,
                        types[i],
                        types[j],
                        mat.occurrences[i][j],
                        mat.matrix[i][j],
                    ],
                )?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use approx::assert_abs_diff_eq;

    fn test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        crate::db::schema::create_indexes(&conn).unwrap();
        conn
    }

    // -----------------------------------------------------------------------
    // build_matrix tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_matrix_simple() {
        // BTreeSet ordering: CH < FF < SL → indices 0, 1, 2
        let types = vec!["CH".to_string(), "FF".to_string(), "SL".to_string()];
        let mut counts: HashMap<(String, String), i64> = HashMap::new();
        counts.insert(("FF".to_string(), "SL".to_string()), 3);
        counts.insert(("FF".to_string(), "CH".to_string()), 7);
        counts.insert(("SL".to_string(), "FF".to_string()), 5);

        let m = build_matrix(&types, &counts);

        // Row CH (idx 0): no data → all zeros
        assert_abs_diff_eq!(m.matrix[0][0], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(m.matrix[0][1], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(m.matrix[0][2], 0.0, epsilon = 1e-10);
        assert_eq!(m.occurrences[0], vec![0i64, 0, 0]);

        // Row FF (idx 1): CH=7, FF=0, SL=3  →  0.7, 0.0, 0.3
        assert_abs_diff_eq!(m.matrix[1][0], 0.7, epsilon = 1e-10);
        assert_abs_diff_eq!(m.matrix[1][1], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(m.matrix[1][2], 0.3, epsilon = 1e-10);
        assert_eq!(m.occurrences[1], vec![7i64, 0, 3]);

        // Row SL (idx 2): SL→FF=5, others=0  →  0.0, 1.0, 0.0  (FF is at col 1)
        assert_abs_diff_eq!(m.matrix[2][0], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(m.matrix[2][1], 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(m.matrix[2][2], 0.0, epsilon = 1e-10);
        assert_eq!(m.occurrences[2], vec![0i64, 5, 0]);
    }

    #[test]
    fn test_build_matrix_row_stochastic() {
        let types = vec!["CH".to_string(), "FF".to_string(), "SL".to_string()];
        let mut counts: HashMap<(String, String), i64> = HashMap::new();
        counts.insert(("FF".to_string(), "SL".to_string()), 3);
        counts.insert(("FF".to_string(), "CH".to_string()), 7);
        counts.insert(("SL".to_string(), "FF".to_string()), 5);

        let m = build_matrix(&types, &counts);

        for (i, row) in m.matrix.iter().enumerate() {
            let row_sum: f64 = row.iter().sum();
            let occ_sum: i64 = m.occurrences[i].iter().sum();
            if occ_sum > 0 {
                assert_abs_diff_eq!(row_sum, 1.0, epsilon = 1e-10);
            } else {
                assert_abs_diff_eq!(row_sum, 0.0, epsilon = 1e-10);
            }
        }
    }

    #[test]
    fn test_build_matrix_empty() {
        let types = vec!["FF".to_string(), "SL".to_string()];
        let counts: HashMap<(String, String), i64> = HashMap::new();

        let m = build_matrix(&types, &counts);

        assert_eq!(m.matrix, vec![vec![0.0f64, 0.0], vec![0.0f64, 0.0]]);
        assert_eq!(m.occurrences, vec![vec![0i64, 0], vec![0i64, 0]]);
    }

    // -----------------------------------------------------------------------
    // compute_pitch_transitions tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_compute_pitch_transitions_empty_db() {
        let conn = test_db();
        let result = compute_pitch_transitions(&conn, 99999, 2024);
        let set = result.expect("should return Ok for missing pitcher");
        assert!(
            set.pitch_types.is_empty(),
            "expected no pitch types for an unknown pitcher"
        );
    }

    #[test]
    fn test_compute_pitch_transitions_with_data() {
        let conn = test_db();

        // Team (required by FK on players)
        conn.execute(
            "INSERT INTO teams (team_id, name, abbreviation) VALUES (1, 'Test Team', 'TST')",
            [],
        )
        .unwrap();

        // Pitcher
        conn.execute(
            "INSERT INTO players (player_id, full_name, primary_position, team_id) \
             VALUES (100, 'Test Pitcher', 'P', 1)",
            [],
        )
        .unwrap();

        // Game on 2024-06-01
        conn.execute(
            "INSERT INTO games (game_pk, game_date, home_team_id, away_team_id, status) \
             VALUES (1, '2024-06-01', 1, 1, 'Final')",
            [],
        )
        .unwrap();

        // Play for pitcher 100
        conn.execute(
            "INSERT INTO plays \
             (id, game_pk, inning, half, event, outs_before, outs_after, \
              bases_before, bases_after, runs_scored, batter_id, pitcher_id) \
             VALUES (1, 1, 1, 'top', 'Strikeout', 0, 1, '---', '---', 0, NULL, 100)",
            [],
        )
        .unwrap();

        // Pitches: FF(1) → SL(2) → FF(3)
        // Transitions: FF→SL (pitch 1→2), SL→FF (pitch 2→3)
        conn.execute(
            "INSERT INTO pitches \
             (id, play_id, pitch_number, pitch_type, count_balls, count_strikes) \
             VALUES (1, 1, 1, 'FF', 0, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO pitches \
             (id, play_id, pitch_number, pitch_type, count_balls, count_strikes) \
             VALUES (2, 1, 2, 'SL', 0, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO pitches \
             (id, play_id, pitch_number, pitch_type, count_balls, count_strikes) \
             VALUES (3, 1, 3, 'FF', 0, 2)",
            [],
        )
        .unwrap();

        let set = compute_pitch_transitions(&conn, 100, 2024)
            .expect("compute should succeed");

        // Both pitch types must be present
        assert!(set.pitch_types.contains(&"FF".to_string()), "expected FF in pitch_types");
        assert!(set.pitch_types.contains(&"SL".to_string()), "expected SL in pitch_types");

        // Locate FF and SL indices in the sorted type list
        let ff_idx = set.pitch_types.iter().position(|t| t == "FF").unwrap();
        let sl_idx = set.pitch_types.iter().position(|t| t == "SL").unwrap();

        // Overall matrix: FF→SL = 1 occurrence, SL→FF = 1 occurrence
        assert_eq!(
            set.overall.occurrences[ff_idx][sl_idx], 1,
            "FF→SL occurrence should be 1"
        );
        assert_eq!(
            set.overall.occurrences[sl_idx][ff_idx], 1,
            "SL→FF occurrence should be 1"
        );

        // Row probabilities for FF (only one target: SL → 1.0)
        assert_abs_diff_eq!(
            set.overall.matrix[ff_idx][sl_idx],
            1.0,
            epsilon = 1e-10
        );
        // Row probabilities for SL (only one target: FF → 1.0)
        assert_abs_diff_eq!(
            set.overall.matrix[sl_idx][ff_idx],
            1.0,
            epsilon = 1e-10
        );

        // by_count must be populated (two transitions → at least one count state)
        assert!(
            !set.by_count.is_empty(),
            "expected at least one count-state bucket"
        );
    }
}
