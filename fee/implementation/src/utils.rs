use warp_fee::state::State;

use crate::contract_utils::js_imports::log;

pub fn is_op(state: &State, address: &str) -> bool {
    address == state.settings.super_operator
        || state.settings.operators.iter().any(|op| op == address)
}

pub fn is_super_op(state: &State, address: &str) -> bool {
    address == state.settings.super_operator
}

fn is_prefix_valid(edition: &str, scarcity: &str) -> bool {
    let edition = edition.parse::<u32>().unwrap_or(0);

    let max_edition = match scarcity {
        "UNIQUE" => 1,
        "LEGENDARY" => 10,
        "EPIC" => 100,
        "RARE" => 1000,
        _ => 0,
    };

    edition > 0 && edition <= max_edition
}

pub fn splited_nft_id(id: &str) -> Option<(&str, &str, &str)> {
    let splited = id.splitn(3, '-').collect::<Vec<&str>>();

    if splited.len() != 3 || !is_prefix_valid(splited[0], splited[1]) {
        None
    } else {
        Some((splited[0], splited[1], splited[2]))
    }
}
