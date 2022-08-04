use astroport::{
    asset::{addr_validate_to_lower, AssetInfo},
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    StdError, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{Config, CONFIG, USER_DCA_ORDERS},
};

/// ## Description
/// Performs a DCA purchase on behalf of another user using the hop route specified.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Params
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the bot who is performing a DCA purchase on behalf of another
/// user, who will be rewarded with a uusd tip.
///
/// * `user` - The address of the user as a [`String`] who is having a DCA purchase fulfilled.
///
/// * `hops` - A [`Vec<SwapOperation>`] of the hop operations to complete in the swap to purchase
/// the target asset.
pub fn perform_dca_purchase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user: String,
    dca_info_id: String,
    hops: Vec<SwapOperation>,
) -> Result<Response, ContractError> {
    let contract_config = CONFIG.load(deps.storage)?;
    // validate user address
    let user_address = addr_validate_to_lower(deps.api, &user)?;
    // retrieve configs
    let user_dca_orders = USER_DCA_ORDERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    let hops_len = hops.len() as u32;

    let tip_cost = contract_config
        .per_hop_fee
        .checked_mul(Uint128::from(hops_len))?;

    sanity_checks(
        &env,
        &contract_config,
        &user_dca_orders,
        dca_info_id.clone(),
        &user_address,
        &hops,
        tip_cost.clone(),
    )?;

    // store messages to send in response
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // load user dca orders and update the relevant one
    USER_DCA_ORDERS.update(
        deps.storage,
        &user_address,
        |orders| -> Result<Vec<DcaInfo>, ContractError> {
            let mut orders = orders.ok_or(ContractError::NonexistentDca {})?;

            for dca in &mut orders {
                if dca.id() == dca_info_id {
                    let order = dca;
                    // retrieve max_spread from user config, or default to contract set max_spread
                    let max_spread = order.max_spread.unwrap_or(contract_config.max_spread);

                    messages = build_messages(
                        &info,
                        &user_address,
                        &contract_config,
                        order,
                        hops.clone(),
                        max_spread,
                        tip_cost.clone(),
                    )?;

                    update_balance(order, &env, tip_cost.clone())?;

                    return Ok(orders);
                }
            }

            Err(ContractError::InvalidInput {
                msg: format!(
                    "The user does not have and DCA order with id = {:?}",
                    dca_info_id
                ),
            })
        },
    )?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "perform_dca_purchase"),
        attr("tip_cost", tip_cost),
    ]))
}

fn sanity_checks(
    env: &Env,
    contract_config: &Config,
    user_dca_orders: &Vec<DcaInfo>,
    dca_info_id: String,
    user: &Addr,
    hops: &Vec<SwapOperation>,
    tip_cost: Uint128,
) -> Result<(), ContractError> {
    if user_dca_orders.is_empty() {
        return Err(ContractError::InvalidInput {
            msg: format!("No DCA orders associated with the user address: {:}", &user),
        });
    }

    let dca_order = user_dca_orders
        .iter()
        .find(|order| order.id() == dca_info_id)
        .ok_or(ContractError::InvalidInput {
            msg: format!(
                "The user does not have and DCA order with id = {:?}",
                dca_info_id
            ),
        })?;

    // validate hops is at least one
    if hops.is_empty() {
        return Err(ContractError::EmptyHopRoute {});
    }

    // validate hops does not exceed max_hops
    let hops_len = hops.len() as u32;
    if hops_len > dca_order.max_hops.unwrap_or(contract_config.max_hops) {
        return Err(ContractError::MaxHopsAssertion { hops: hops_len });
    }

    // validate purchaser has enough funds to pay the sender
    if tip_cost > dca_order.balance.tip.amount {
        return Err(ContractError::InsufficientTipBalance {});
    }

    // check that first hoDecimalp is target asset
    let first_hop = match &hops[0] {
        SwapOperation::NativeSwap { offer_denom, .. } => AssetInfo::NativeToken {
            denom: offer_denom.clone(),
        },
        SwapOperation::AstroSwap {
            offer_asset_info, ..
        } => offer_asset_info.clone(),
    };

    if first_hop != dca_order.balance.deposit.info {
        return Err(ContractError::StartAssetAssertion {});
    }

    // check that last hop is target asset
    let last_hop = &hops
        .last()
        .ok_or(ContractError::EmptyHopRoute {})?
        .get_target_asset_info();
    if last_hop != &dca_order.balance.target.info {
        return Err(ContractError::TargetAssetAssertion {});
    }

    // check that it has been long enough between dca purchases
    if dca_order.balance.last_purchase + dca_order.interval > env.block.time.seconds() {
        return Err(ContractError::PurchaseTooEarly {});
    }

    Ok(())
}

fn update_balance(order: &mut DcaInfo, env: &Env, tip_cost: Uint128) -> Result<(), ContractError> {
    // subtract dca_amount from order and update last_purchase time
    order.balance.deposit.amount = order
        .balance
        .deposit
        .amount
        .checked_sub(order.dca_amount.amount)
        .map_err(|_| ContractError::InsufficientBalance {})?;

    order.balance.spent.amount = order
        .balance
        .spent
        .amount
        .checked_add(order.dca_amount.amount)
        .map_err(|_| ContractError::BalanceUpdateError {
            msg: "Unable to add dca_amount to the spent amount".to_string(),
        })?;

    order.balance.tip.amount = order
        .balance
        .tip
        .amount
        .checked_sub(tip_cost)
        .map_err(|_| ContractError::InsufficientBalance {})?;

    order.balance.last_purchase = env.block.time.seconds();

    Ok(())
}

fn build_messages(
    info: &MessageInfo,
    user_address: &Addr,
    contract_config: &Config,
    order: &mut DcaInfo,
    hops: Vec<SwapOperation>,
    max_spread: Decimal,
    tip_cost: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = Vec::new();
    // add funds and router message to response
    if let AssetInfo::Token { contract_addr } = &order.balance.deposit.info {
        // send a TransferFrom request to the token to the router
        messages.push(
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: user_address.to_string(),
                    recipient: contract_config.router_addr.to_string(),
                    amount: order.dca_amount.amount,
                })?,
            }
            .into(),
        );
    }

    // if it is a native token, we need to send the funds
    let funds = match &order.balance.deposit.info {
        AssetInfo::NativeToken { denom } => vec![Coin {
            amount: order.dca_amount.amount,
            denom: denom.clone(),
        }],
        AssetInfo::Token { .. } => vec![],
    };

    // tell the router to perform swap operations
    messages.push(
        WasmMsg::Execute {
            contract_addr: contract_config.router_addr.to_string(),
            funds,
            msg: to_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                operations: hops,
                minimum_receive: None,
                to: Some(user_address.clone()),
                max_spread: Some(max_spread),
            })?,
        }
        .into(),
    );

    // add tip payment to messages
    match &order.balance.tip.info {
        AssetInfo::NativeToken { denom } => messages.push(
            BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    amount: tip_cost,
                    denom: denom.to_string(),
                }],
            }
            .into(),
        ),
        AssetInfo::Token { contract_addr } => todo!(),
    }

    Ok(messages)
}
