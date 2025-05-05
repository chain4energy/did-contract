// use cosmwasm_std::{
//     entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
// };

pub mod contract;
pub mod state;
pub mod error;
mod multiset;

#[cfg(test)]
mod test;
// mod msg;

// #[entry_point]
// pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: Empty)
//   -> StdResult<Response>
// {
//     contract::instantiate(deps, env, info, msg)
// }

// #[entry_point]
// pub fn query(deps: Deps, env: Env, msg: msg::QueryMsg)
//   -> StdResult<Binary>
// {
//     contract::query(deps, env, msg)
// }