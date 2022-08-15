use crate::state::DCA_ORDERS;
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{Deps, StdResult};

/// ## Description
/// Returns the information of a particular dca order specified by an id
///
/// The result is returned in a [`DcaInfo`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `id` - the dca order id given as a [`String`].
pub fn get_dca_orders(deps: Deps, id: String) -> StdResult<DcaInfo> {
    return DCA_ORDERS.load(deps.storage, id);
}
