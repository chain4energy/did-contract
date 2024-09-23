use serde::{Deserialize, Serialize};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum QueryMsg {
    Greet {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GreetResp {
    pub message: String,
}