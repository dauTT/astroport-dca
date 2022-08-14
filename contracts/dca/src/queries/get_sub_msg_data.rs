use cosmwasm_std::{Deps, StdResult, SubMsgResponse};

use crate::state::SUB_MSG_DATA;

pub fn get_sub_msg_data(deps: Deps) -> StdResult<SubMsgResponse> {
    return SUB_MSG_DATA.load(deps.storage);
}
