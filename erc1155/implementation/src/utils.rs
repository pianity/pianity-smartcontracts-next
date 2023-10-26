use crate::state::KvState;

pub async fn is_op(address: &str) -> bool {
    KvState::settings()
        .operators()
        .get()
        .await
        .contains(&address.into())
    // true
    // is_super_op(state, address) || state.settings.operators.contains(&address.into())
}

pub async fn is_super_op(address: &str) -> bool {
    KvState::settings()
        .super_operators()
        .get()
        .await
        .contains(&address.into())
    // state.settings.super_operators.contains(&address.into())
}
