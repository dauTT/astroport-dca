use astroport::asset::addr_validate_to_lower;
use cosmwasm_std::{Deps, StdResult};

use crate::state::DCA_ORDERS;

use astroport_dca::dca::DcaInfo;

/// ## Description
/// Returns the configuration set for a user to override the default contract configuration.
///
/// The result is returned in a [`UserConfig`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `user` - The users lowercase address as a [`String`].
pub fn get_dca_orders(deps: Deps, id: String) -> StdResult<DcaInfo> {
    return DCA_ORDERS.load(deps.storage, id);
}
