mod action;
pub mod actions;
mod contract;
pub mod contract_utils;
mod error;
pub mod state;
mod utils;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Serialize;

    use crate::state::{Balance, State, Token};

    #[test]
    fn serialize_test() {
        let mut state = State::default();

        state.tokens.insert(
            "a".to_string(),
            Token {
                ticker: "a".to_string(),
                balances: HashMap::from([("address".to_string(), Balance::new(1))]),
            },
        );

        let serialized = serde_json::to_string_pretty(&state).unwrap();

        let deserialized: State = serde_json::from_str(&serialized).unwrap();

        println!("SERIALIZE {}", serialized);
        println!("DESERIALIZE {:?}", deserialized);
    }
}
