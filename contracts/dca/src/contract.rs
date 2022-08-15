use crate::error::ContractError;
use crate::handlers::{
    cancel_dca_order, create_dca_order, deposit, modify_dca_order, perform_dca_purchase,
    update_config, withdraw, ModifyDcaOrderParameters,
};
use crate::queries::{get_config, get_dca_orders, get_sub_msg_data, get_user_dca_orders};
use crate::state::{Config, CONFIG, SUB_MSG_DATA, TMP_GAS_BALANCE_AND_TIP_COST};
use astroport::asset::addr_validate_to_lower;
use astroport_dca::dca::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsgResponse,
};
use cw2::set_contract_version;
use std::str::FromStr;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "astroport-dca";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in [`InstantiateMsg`].
///
/// Returns a [`Response`] with the specified attributes if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `_env` - The [`Env`] of the blockchain.
///
/// * `_info` - The [`MessageInfo`] from the contract instantiator.
///
/// * `msg` - A [`InstantiateMsg`] which contains the parameters for creating the contract.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = addr_validate_to_lower(deps.api, &msg.owner)?;
    // get max spread in decimal form
    let max_spread = Decimal::from_str(&msg.max_spread)?;

    // validate that factory_addr and router_addr is an address
    let factory_addr = addr_validate_to_lower(deps.api, &msg.factory_addr)?;
    let router_addr = addr_validate_to_lower(deps.api, &msg.router_addr)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // TODO:  make sure there a no duplicated in msg.whitelisted_tokens
    let config = Config {
        owner,
        max_hops: msg.max_hops,
        max_spread,
        gas_info: msg.gas_info,
        per_hop_fee: msg.per_hop_fee,
        whitelisted_tokens: msg.whitelisted_tokens,
        factory_addr,
        router_addr,
    };

    CONFIG.save(deps.storage, &config)?;
    TMP_GAS_BALANCE_AND_TIP_COST.save(deps.storage, &None)?;

    let r: SubMsgResponse = SubMsgResponse {
        data: None,
        events: vec![],
    };

    SUB_MSG_DATA.save(deps.storage, &r)?;

    Ok(Response::new())
}

/// ## Description
/// Used for contract migration. Returns a default object of type [`Response`].
/// ## Arguments
/// * `_deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `_env` - The [`Env`] of the blockchain.
///
/// * `_msg` - The [`MigrateMsg`] to migrate the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

/// ## Description
/// Exposes all the execute functions available in the contract.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] that contains the message information.
///
/// * `msg` - The [`ExecuteMsg`] to run.
///
/// ## Execution Messages
/// * **ExecuteMsg::Deposit {
///          deposit_type,
///          dca_order_id,
///          asset } Deposit an asset (source/tip/gas) into the dca contract.
///
/// * **ExecuteMsg::UpdateConfig {
///         max_hops,
///         per_hop_fee,
///         whitelisted_tokens_source,
///         whitelisted_tokens_tip,
///         max_spread,
///         router_addr
///     }** Updates the contract configuration with the specified input parameters.
///
/// * **ExecuteMsg::CreateDcaOrder {
///         start_at,
///         interval,
///         dca_amount,
///         max_hops,
///         max_spread,
///         source,
///         tip,
///         gas,
///         target_info
///     }** Creates a new DCA order where `source` will purchase the `target_info` asset.
///
/// * **ExecuteMsg::Withdraw {
///         withdraw_type,
///         dca_order_id,
///         asset} withdraw an asset (source/tip/gas/target) from the dca contract.
///
/// * **ExecuteMsg::PerformDcaPurchase { dca_order_id, hops }** Performs a DCA purchase on behalf of a
/// specified user given a hop route.
///
/// * **ExecuteMsg::CancelDcaOrder { initial_asset }** Cancels an existing DCA order.
///
/// * **ExecuteMsg::ModifyDcaOrder {
///         id,
///         new_source_asset,
///         new_target_asset_info,
///         new_tip_asset,
///         new_interval,
///         new_dca_amount,
///         new_start_at,
///         new_max_hops,
///         new_max_spread,
///     }** Modifies an existing DCA order, allowing the user to change certain parameters.
///
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {
            deposit_type,
            dca_order_id,
            asset,
        } => deposit(deps, env, info, deposit_type, dca_order_id, asset),

        ExecuteMsg::UpdateConfig {
            max_hops,
            per_hop_fee,
            whitelisted_tokens_source,
            whitelisted_tokens_tip,
            max_spread,
            router_addr,
        } => update_config(
            deps,
            info,
            max_hops,
            per_hop_fee,
            whitelisted_tokens_source,
            whitelisted_tokens_tip,
            max_spread,
            router_addr,
        ),
        ExecuteMsg::CreateDcaOrder {
            start_at,
            interval,
            dca_amount,
            max_hops,
            max_spread,
            source,
            tip,
            gas,
            target_info,
        } => create_dca_order(
            deps,
            env,
            info,
            start_at,
            interval,
            dca_amount,
            max_hops,
            max_spread,
            source,
            tip,
            gas,
            target_info,
        ),
        ExecuteMsg::Withdraw {
            withdraw_type,
            dca_order_id,
            asset,
        } => withdraw(deps, info, withdraw_type, dca_order_id, asset),
        ExecuteMsg::PerformDcaPurchase { dca_order_id, hops } => {
            perform_dca_purchase(deps, env, info, dca_order_id, hops)
        }
        ExecuteMsg::CancelDcaOrder { id } => cancel_dca_order(deps, info, id),
        ExecuteMsg::ModifyDcaOrder {
            id,
            new_source_asset,
            new_target_asset_info,
            new_tip_asset,
            new_interval,
            new_dca_amount,
            new_start_at,
            new_max_hops,
            new_max_spread,
        } => modify_dca_order(
            deps,
            env,
            info,
            id,
            ModifyDcaOrderParameters {
                new_source_asset,
                new_target_asset_info,
                new_tip_asset,
                new_interval,
                new_dca_amount,
                new_start_at,
                new_max_hops,
                new_max_spread,
            },
        ),
    }
}

/// ## Description
/// Exposes all the queries available in the contract.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `msg` - The [`QueryMsg`] to run.
///
/// ## Queries
/// * **QueryMsg::Config {}** Returns information about the configuration of the contract in a
/// [`Config`] object.
///
/// * **QueryMsg::UserDcaOrders {}** Returns the list of dca order ids for a specified user. The list  
/// is given by the [`Vec<String>`] object.
///
/// * **QueryMsg::DcaOrders {id}** Returns information about the dca order with a specific id. The information
/// is encapsulated in a [`DcaInfo`] object.
///
/// * **QueryMsg::ReplySubMsgResponse {}** Returns information about the reply of the swap operation. The information
/// is encapsulated in a [`SubMsgResponse`] object.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&get_config(deps)?),
        QueryMsg::UserDcaOrders { user } => to_binary(&get_user_dca_orders(deps, user)?),
        QueryMsg::DcaOrders { id } => to_binary(&get_dca_orders(deps, id)?),
        QueryMsg::ReplySubMsgResponse {} => to_binary(&get_sub_msg_data(deps)?),
    }
}
