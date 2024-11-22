use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, HexBinary, MessageInfo, Response,
    StdResult,
};
use quartz_common::contract::handler::RawHandler;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{ContractState, STATE, STATE_PK},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Handle Quartz initialization
    msg.quartz.handle_raw(deps.branch(), &env, &info)?;

    // Initialize STATE and STATE_PK
    let state: HexBinary = HexBinary::from(&[0x00]);
    STATE.save(deps.storage, &state)?;

    let state = ContractState {
        public_keys: Vec::new(),
    };
    STATE_PK.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Handle Quartz messages
        ExecuteMsg::Quartz(msg) => match msg.clone() {
            quartz_common::contract::msg::RawExecuteMsg::RawSessionCreate(_raw_attested) => {
                msg.handle_raw(deps, &env, &info).map_err(Into::into)
            }
            quartz_common::contract::msg::RawExecuteMsg::RawSessionSetPubKey(_raw_attested) => {
                let resp = msg
                    .handle_raw(deps.branch(), &env, &info)
                    .map_err(Into::into);
                
                let attr = resp.as_ref().unwrap().attributes.clone();

                let mut state = STATE_PK.load(deps.storage)?;
                let new_pub_key_value = &attr[1].value;
                
                let _r = state.public_keys.push(new_pub_key_value.to_string());
                
                STATE_PK.save(deps.storage, &state)?;
                
                return resp;
            }
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPublicKeys {} => to_json_binary(&query::get_public_keys(deps)?),
    }
}

mod query {

    use crate::state::STATE_PK;
    use cosmwasm_std::{Deps, StdResult};

    pub fn get_public_keys(deps: Deps) -> StdResult<Vec<String>> {
        let state = STATE_PK.load(deps.storage)?;
        Ok(state.public_keys)
    }
}
