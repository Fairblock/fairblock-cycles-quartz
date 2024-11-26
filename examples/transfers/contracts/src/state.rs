use cosmwasm_std::HexBinary;
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContractState {
    pub public_keys: Vec<String>,
}

pub const STATE_PK: Item<ContractState> = Item::new("state_pk");
pub const STATE: Item<HexBinary> = Item::new("state");


