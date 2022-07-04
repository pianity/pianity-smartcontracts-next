use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HandlerResult<State, ReadResponse> {
    Write(State),
    Read(State, ReadResponse),
}
