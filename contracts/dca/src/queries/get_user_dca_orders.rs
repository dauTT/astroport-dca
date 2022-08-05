use astroport::asset::{addr_validate_to_lower, AssetInfo};
use astroport_dca::dca::DcaQueryInfo;
use cosmwasm_std::{Deps, Env, StdResult};

use crate::{get_token_allowance::get_token_allowance, state::USER_DCA_ORDERS};

/// ## Description
/// Returns a users DCA orders currently set.
///
/// The result is returned in a [`Vec<DcaQueryInfo`] object of the users current DCA orders with the
/// `amount` of each order set to the native token amount that can be spent, or the token allowance.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `user` - The users lowercase address as a [`String`].
pub fn get_user_dca_orders(deps: Deps, user: String) -> StdResult<Vec<String>> {
    let user_address = addr_validate_to_lower(deps.api, &user)?;
    return USER_DCA_ORDERS.load(deps.storage, &user_address);
}
