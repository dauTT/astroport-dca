use crate::state::SUB_MSG_DATA;
use cosmwasm_std::{Deps, StdResult, SubMsgResponse};

/// ## Description
/// Returns the Reply object from the perform_dca_purchase operation.
///
/// The result is returned in a [`Vec<SubMsgResponse>`] object and it is used
/// internally for debugging purposes.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
pub fn get_sub_msg_data(deps: Deps) -> StdResult<SubMsgResponse> {
    return SUB_MSG_DATA.load(deps.storage);
}
