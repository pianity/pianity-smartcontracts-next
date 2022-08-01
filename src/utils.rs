use crate::state::State;

pub fn is_op(state: &State, address: &str) -> bool {
    address == state.settings.super_operator
        || state.settings.operators.iter().any(|op| op == address)
}

pub fn is_super_op(state: &State, address: &str) -> bool {
    address == state.settings.super_operator
}
