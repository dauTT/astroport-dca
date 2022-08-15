use crate::state::USER_DCA_ORDERS;
use astroport::asset::addr_validate_to_lower;
use cosmwasm_std::{Deps, StdResult};

/// ## Description
/// Returns a list of DCA order ids which the user owns.
///
/// The result is returned in a [`Vec<String>`] object of the users current DCA orders
/// where the dca order id is tracked as a String.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `user` - The users lowercase address as a [`String`].
pub fn get_user_dca_orders(deps: Deps, user: String) -> StdResult<Vec<String>> {
    let user_address = addr_validate_to_lower(deps.api, &user)?;
    return USER_DCA_ORDERS.load(deps.storage, &user_address);
}
