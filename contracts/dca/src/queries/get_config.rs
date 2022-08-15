use crate::state::{Config, CONFIG};
use cosmwasm_std::{Deps, StdResult};

/// ## Description
/// Returns the contract configuration of the dca contract.
///
/// The result is returned in a [`Config`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
pub fn get_config(deps: Deps) -> StdResult<Config> {
    return CONFIG.load(deps.storage);
}
