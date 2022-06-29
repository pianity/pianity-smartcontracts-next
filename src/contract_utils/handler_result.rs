use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HandlerResult<QueryResponseMsg> {
    Write,
    Read(QueryResponseMsg),
}
