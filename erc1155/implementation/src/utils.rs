use warp_erc1155::state::State;

pub fn is_op(state: &State, address: &str) -> bool {
    true
    // is_super_op(state, address) || state.settings.operators.contains(&address.into())
}

pub fn is_super_op(state: &State, address: &str) -> bool {
    true
    // state.settings.super_operators.contains(&address.into())
}
