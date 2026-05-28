use super::states::ACTIVE_STATES;
use super::transitions::TransitionMatrix;

pub struct ExpectedRuns {
    pub states: Vec<String>,
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

pub fn compute_expected_runs(tm: &TransitionMatrix) -> ExpectedRuns {
    let n = ACTIVE_STATES.len(); // 24 active states
    let total = tm.states.len(); // 25 total (including absorbing)

    // Q = 24x24 submatrix of transitions among active states
    let mut q = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            q[i][j] = tm.matrix[i][j];
        }
    }

    // R_i = expected runs scored per visit to state i
    // = sum over all j of P(i->j) * avg_runs(i->j) for ALL transitions (including absorbing)
    let mut r = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..total {
            r[i] += tm.matrix[i][j] * tm.runs[i][j];
        }
    }

    // N = (I - Q)^-1 (fundamental matrix)
    let mut im_q = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            im_q[i][j] = if i == j { 1.0 - q[i][j] } else { -q[i][j] };
        }
    }

    let n_matrix = invert_matrix(&im_q);

    // Expected runs from state i = sum_j N[i][j] * R[j]
    let mut expected = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..n {
            expected[i] += n_matrix[i][j] * r[j];
        }
    }

    let states: Vec<String> = ACTIVE_STATES.iter().map(|s| s.to_string()).collect();
    let labels: Vec<String> = ACTIVE_STATES
        .iter()
        .map(|s| super::states::state_label(s))
        .collect();

    ExpectedRuns {
        states,
        labels,
        values: expected,
    }
}

/// Gaussian elimination with partial pivoting to invert an n x n matrix.
pub(crate) fn invert_matrix(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = m.len();
    // Augmented matrix [M | I]
    let mut aug = vec![vec![0.0f64; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = m[i][j];
        }
        aug[i][n + i] = 1.0;
    }

    // Forward elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        if pivot.abs() < 1e-12 {
            eprintln!("[mlb-markov] Near-singular matrix at col {}, zeroing row", col);
            for j in 0..(2 * n) {
                aug[col][j] = 0.0;
            }
            continue;
        }

        // Scale pivot row
        for j in 0..(2 * n) {
            aug[col][j] /= pivot;
        }

        // Eliminate column in all other rows
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            for j in 0..(2 * n) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Extract inverse from right half
    let mut inv = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            inv[i][j] = aug[i][n + j];
        }
    }
    inv
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::states::ALL_STATES;
    use approx::assert_abs_diff_eq;

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = a.len();
        let mut c = vec![vec![0.0f64; n]; n];
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    c[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        c
    }

    fn assert_identity(m: &[Vec<f64>], eps: f64) {
        let n = m.len();
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert_abs_diff_eq!(m[i][j], expected, epsilon = eps);
            }
        }
    }

    /// Build a 25-state TransitionMatrix where every active state transitions
    /// to the absorbing state (index 24) with probability 1.0 and 0 runs.
    /// The absorbing state self-loops at probability 1.0.
    fn make_tm(n: usize) -> TransitionMatrix {
        assert_eq!(n, 25, "make_tm only supports 25-state matrices");
        let states: Vec<String> = ALL_STATES.iter().map(|s| s.to_string()).collect();
        let mut matrix = vec![vec![0.0f64; n]; n];
        let runs = vec![vec![0.0f64; n]; n];
        let mut counts = vec![vec![0i64; n]; n];

        // Each active state (0..24) → absorbing (24)
        for i in 0..24 {
            matrix[i][24] = 1.0;
            counts[i][24] = 1;
        }
        // Absorbing state self-loops
        matrix[24][24] = 1.0;
        counts[24][24] = 1;

        TransitionMatrix { states, matrix, runs, counts }
    }

    // ---------------------------------------------------------------------------
    // Matrix inversion tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_invert_2x2_identity() {
        let identity = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let inv = invert_matrix(&identity);
        assert_abs_diff_eq!(inv[0][0], 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(inv[0][1], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(inv[1][0], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(inv[1][1], 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_invert_2x2_known() {
        // A = [[4,7],[2,6]], det = 4*6 - 7*2 = 10
        // A^-1 = (1/10) * [[6,-7],[-2,4]] = [[0.6,-0.7],[-0.2,0.4]]
        let a = vec![vec![4.0, 7.0], vec![2.0, 6.0]];
        let inv = invert_matrix(&a);
        let product = mat_mul(&a, &inv);
        assert_identity(&product, 1e-10);
        assert_abs_diff_eq!(inv[0][0],  0.6, epsilon = 1e-10);
        assert_abs_diff_eq!(inv[0][1], -0.7, epsilon = 1e-10);
        assert_abs_diff_eq!(inv[1][0], -0.2, epsilon = 1e-10);
        assert_abs_diff_eq!(inv[1][1],  0.4, epsilon = 1e-10);
    }

    #[test]
    fn test_invert_3x3_known() {
        // A = [[1,2,3],[0,1,4],[5,6,0]]
        // A^-1 = [[-24,18,5],[20,-15,-4],[-5,4,1]]
        let a = vec![
            vec![1.0, 2.0, 3.0],
            vec![0.0, 1.0, 4.0],
            vec![5.0, 6.0, 0.0],
        ];
        let inv = invert_matrix(&a);
        let product = mat_mul(&a, &inv);
        assert_identity(&product, 1e-10);

        let expected = vec![
            vec![-24.0, 18.0,  5.0],
            vec![ 20.0, -15.0, -4.0],
            vec![ -5.0,  4.0,  1.0],
        ];
        for i in 0..3 {
            for j in 0..3 {
                assert_abs_diff_eq!(inv[i][j], expected[i][j], epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn test_invert_4x4_diagonally_dominant() {
        let a = vec![
            vec![10.0, 1.0, 2.0, 0.0],
            vec![ 1.0, 10.0, 0.0, 1.0],
            vec![ 0.0,  2.0, 10.0, 1.0],
            vec![ 1.0,  0.0,  1.0, 10.0],
        ];
        let inv = invert_matrix(&a);
        let product = mat_mul(&a, &inv);
        assert_identity(&product, 1e-8);
    }

    #[test]
    fn test_invert_near_singular() {
        // Singular matrix — should not panic; the code zeros the row instead.
        let singular = vec![vec![1.0, 0.0], vec![0.0, 0.0]];
        let result = std::panic::catch_unwind(|| invert_matrix(&singular));
        assert!(result.is_ok(), "invert_matrix panicked on near-singular input");
    }

    // ---------------------------------------------------------------------------
    // Expected-runs tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_expected_runs_all_absorbing() {
        // All active states → absorbing with 0 runs → every RE value = 0.0
        let tm = make_tm(25);
        let er = compute_expected_runs(&tm);
        assert_eq!(er.values.len(), 24);
        for (i, &v) in er.values.iter().enumerate() {
            assert!(
                (v - 0.0f64).abs() < 1e-10,
                "state {} ({}) expected 0.0, got {}",
                i, er.states[i], v
            );
        }
    }

    #[test]
    fn test_expected_runs_single_state_scores() {
        // State 0 (0_000) → absorbing scoring 1 run; all others score 0.
        let mut tm = make_tm(25);
        // runs[0][24] = 1.0 (already matrix[0][24] = 1.0 from make_tm)
        tm.runs[0][24] = 1.0;

        let er = compute_expected_runs(&tm);
        assert_eq!(er.values.len(), 24);
        assert_abs_diff_eq!(er.values[0], 1.0, epsilon = 1e-10);
        for i in 1..24 {
            assert!(
                (er.values[i] - 0.0f64).abs() < 1e-10,
                "state {} should be 0.0, got {}",
                i, er.values[i]
            );
        }
    }

    #[test]
    fn test_expected_runs_chain() {
        // State 0 → state 1 (prob 1.0, 0.5 runs), state 1 → absorbing (prob 1.0, 0.3 runs).
        // Expected: values[0] = 0.8, values[1] = 0.3.
        let mut tm = make_tm(25);

        // Clear state 0's direct absorbing transition
        tm.matrix[0][24] = 0.0;
        tm.runs[0][24] = 0.0;
        tm.counts[0][24] = 0;

        // State 0 → state 1
        tm.matrix[0][1] = 1.0;
        tm.runs[0][1] = 0.5;
        tm.counts[0][1] = 1;

        // State 1 still → absorbing (from make_tm), set runs
        tm.runs[1][24] = 0.3;

        let er = compute_expected_runs(&tm);
        assert_abs_diff_eq!(er.values[0], 0.8, epsilon = 1e-10);
        assert_abs_diff_eq!(er.values[1], 0.3, epsilon = 1e-10);
    }

    #[test]
    fn test_expected_runs_re24_sanity() {
        // Simplified realistic TM for state 0 (0 outs, bases empty).
        // Verify compute_expected_runs doesn't panic and produces a positive
        // value below 5.0 — a sanity bound for an empty-base, 0-out situation.
        let mut tm = make_tm(25);

        // Clear state 0's default absorbing transition
        tm.matrix[0][24] = 0.0;
        tm.counts[0][24] = 0;

        // 0_000 → 1_000 (idx 8): out, no advance, prob 0.55
        tm.matrix[0][8] = 0.55;
        tm.runs[0][8] = 0.0;
        tm.counts[0][8] = 55;

        // 0_000 → 0_100 (idx 1): single, prob 0.20
        tm.matrix[0][1] = 0.20;
        tm.runs[0][1] = 0.0;
        tm.counts[0][1] = 20;

        // 0_000 → 0_010 (idx 2): double, prob 0.05
        tm.matrix[0][2] = 0.05;
        tm.runs[0][2] = 0.0;
        tm.counts[0][2] = 5;

        // 0_000 → 0_001 (idx 3): triple, prob 0.05
        tm.matrix[0][3] = 0.05;
        tm.runs[0][3] = 0.0;
        tm.counts[0][3] = 5;

        // 0_000 → absorbing (idx 24): solo HR, prob 0.04, 1 run
        tm.matrix[0][24] = 0.04;
        tm.runs[0][24] = 1.0;
        tm.counts[0][24] = 4;

        // 0_000 → 0_100 already set; add walk by routing remaining prob to 0_100
        // Total so far: 0.55 + 0.20 + 0.05 + 0.05 + 0.04 = 0.89
        // Walk (0_000 → 0_100): prob 0.11 — add to existing matrix[0][1]
        tm.matrix[0][1] += 0.11;
        tm.counts[0][1] += 11;
        // matrix[0][1] is now 0.31; total row = 1.00

        let er = compute_expected_runs(&tm);
        assert_eq!(er.values.len(), 24);
        let v = er.values[0];
        assert!(v > 0.0, "RE(0_000) should be positive, got {}", v);
        assert!(v < 5.0, "RE(0_000) should be < 5.0, got {}", v);
    }
}
