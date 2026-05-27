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
