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
fn invert_matrix(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
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
            // Near-singular; leave row as-is (shouldn't happen for valid Markov chains)
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
