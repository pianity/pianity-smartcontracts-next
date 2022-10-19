use warp_packs::state::{PackScarcity, State};

fn index_to_editions_count(n: usize) -> u32 {
    (0..n).fold(1, |acc, _| acc * 10)
}

pub fn get_all_nfts_ids(nfts: &PackScarcity) -> Vec<String> {
    Vec::from(nfts)
        .iter()
        .enumerate()
        .flat_map(|(i, id)| {
            (0..index_to_editions_count(i)).map(move |edition| format!("{}-{}", edition + 1, id))
        })
        .collect()
}

pub fn is_op(state: &State, address: &str) -> bool {
    is_super_op(state, address) || state.settings.operators.contains(&address.into())
}

pub fn is_super_op(state: &State, address: &str) -> bool {
    state.settings.super_operators.contains(&address.into())
}
