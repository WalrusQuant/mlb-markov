pub const ABSORBING_STATE: &str = "3_---";

pub const ACTIVE_STATES: [&str; 24] = [
    "0_000", "0_100", "0_010", "0_001", "0_110", "0_101", "0_011", "0_111",
    "1_000", "1_100", "1_010", "1_001", "1_110", "1_101", "1_011", "1_111",
    "2_000", "2_100", "2_010", "2_001", "2_110", "2_101", "2_011", "2_111",
];

pub const ALL_STATES: [&str; 25] = [
    "0_000", "0_100", "0_010", "0_001", "0_110", "0_101", "0_011", "0_111",
    "1_000", "1_100", "1_010", "1_001", "1_110", "1_101", "1_011", "1_111",
    "2_000", "2_100", "2_010", "2_001", "2_110", "2_101", "2_011", "2_111",
    "3_---",
];

pub fn state_index(state: &str) -> Option<usize> {
    ALL_STATES.iter().position(|&s| s == state)
}

pub fn encode_state(outs: i32, bases: &str) -> String {
    if outs >= 3 {
        ABSORBING_STATE.to_string()
    } else {
        format!("{}_{}", outs, bases)
    }
}

pub fn state_label(state: &str) -> String {
    if state == ABSORBING_STATE {
        return "3 Outs (End)".to_string();
    }

    let parts: Vec<&str> = state.split('_').collect();
    if parts.len() != 2 {
        return state.to_string();
    }

    let outs = parts[0];
    let bases = parts[1];

    let base_desc = match bases {
        "000" => "Empty",
        "100" => "1B",
        "010" => "2B",
        "001" => "3B",
        "110" => "1B 2B",
        "101" => "1B 3B",
        "011" => "2B 3B",
        "111" => "Loaded",
        _ => bases,
    };

    format!("{} out, {}", outs, base_desc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_states_count() {
        assert_eq!(ALL_STATES.len(), 25);
        assert_eq!(ACTIVE_STATES.len(), 24);
    }

    #[test]
    fn test_absorbing_state_is_last() {
        assert_eq!(ALL_STATES[24], ABSORBING_STATE);
        assert!(!ACTIVE_STATES.contains(&ABSORBING_STATE));
    }

    #[test]
    fn test_encode_state_normal() {
        assert_eq!(encode_state(0, "000"), "0_000");
        assert_eq!(encode_state(1, "100"), "1_100");
        assert_eq!(encode_state(2, "111"), "2_111");
    }

    #[test]
    fn test_encode_state_absorbing() {
        assert_eq!(encode_state(3, "000"), "3_---");
        assert_eq!(encode_state(4, "000"), "3_---");
        assert_eq!(encode_state(100, "000"), "3_---");
    }

    #[test]
    fn test_state_index_all_found() {
        for (i, &state) in ALL_STATES.iter().enumerate() {
            assert_eq!(state_index(state), Some(i));
        }
    }

    #[test]
    fn test_state_index_unknown() {
        assert_eq!(state_index("4_000"), None);
        assert_eq!(state_index(""), None);
        assert_eq!(state_index("invalid"), None);
        assert_eq!(state_index("0_999"), None);
    }

    #[test]
    fn test_state_label_examples() {
        assert_eq!(state_label("0_000"), "0 out, Empty");
        assert_eq!(state_label("1_100"), "1 out, 1B");
        assert_eq!(state_label("2_111"), "2 out, Loaded");
        assert_eq!(state_label("0_011"), "0 out, 2B 3B");
        assert_eq!(state_label("3_---"), "3 Outs (End)");
    }

    #[test]
    fn test_state_label_malformed() {
        assert_eq!(state_label(""), "");
        assert_eq!(state_label("no_under_score"), "no_under_score");
    }
}
