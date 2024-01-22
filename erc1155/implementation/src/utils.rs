use crate::state::State;

pub async fn is_op(address: &str) -> bool {
    State::settings()
        .operators()
        .get()
        .await
        .contains(&address.into())
        || State::settings()
            .super_operators()
            .get()
            .await
            .contains(&address.into())
}

pub async fn is_super_op(address: &str) -> bool {
    State::settings()
        .super_operators()
        .get()
        .await
        .contains(&address.into())
}
