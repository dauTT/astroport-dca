// use std::clone;

use std::str::FromStr;

use crate::utils::query_asset_balance;
use astroport::asset::Asset;
use astroport::{
    asset::AssetInfo,
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, Event,
    MessageInfo, Reply, ReplyOn, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use schemars::Map;

use crate::{
    error::ContractError,
    state::{Config, CONFIG, DCA_ORDERS, SUB_MSG_DATA, TMP_GAS_BALANCE_AND_TIP_COST},
};

/// A `reply` call code ID of sub-message.
const PERFORM_DCA_PURCHASE_REPLY_ID: u64 = 1;

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
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dca_order_id: String,
    hops: Vec<SwapOperation>,
) -> Result<Response, ContractError> {
    let contract_config = CONFIG.load(deps.storage)?;
    let order = DCA_ORDERS.load(deps.as_ref().storage, dca_order_id.clone())?;
    let hops_len = hops.len() as u32;
    let tip_cost = contract_config
        .per_hop_fee
        .checked_mul(Uint128::from(hops_len))?;
    let tip_cost_asset = Asset {
        info: order.balance.tip.info.clone(),
        amount: tip_cost.clone(),
    };

    // Store the contract gas balance before the purchase and the tip cost
    // This information will be used later to calculated the user gas fee.
    store_contract_gas_balance_and_tip_cost(
        &mut deps,
        env.clone(),
        order.clone(),
        tip_cost_asset.clone(),
    )?;
    sanity_checks(&env, &contract_config, &order, &hops, tip_cost.clone())?;

    // store messages to send in response
    let mut messages: Vec<CosmosMsg> = vec![];
    let mut sub_messages: Vec<SubMsg> = vec![];
    // load dca order and update it
    DCA_ORDERS.update(
        deps.storage,
        dca_order_id.clone(),
        |order| -> Result<DcaInfo, ContractError> {
            let order = &mut order.unwrap();

            // retrieve max_spread from user config, or default to contract set max_spread
            let max_spread = order.max_spread.unwrap_or(contract_config.max_spread);

            (sub_messages, messages) = build_messages(
                &env,
                &info,
                &contract_config,
                order,
                hops.clone(),
                max_spread,
                tip_cost_asset.clone(),
            )?;

            update_balance(order, &env, tip_cost_asset.clone())?;

            return Ok(order.clone());
        },
    )?;

    Ok(Response::new()
        .add_messages(messages)
        .add_submessages(sub_messages)
        .add_attributes(vec![
            attr("action", "perform_dca_purchase"),
            attr("dca_order_id", dca_order_id),
            attr("hops", format!("{:?}", hops)),
        ]))
}

pub fn store_contract_gas_balance_and_tip_cost(
    deps: &mut DepsMut,
    env: Env,
    order: DcaInfo,
    tip_cost: Asset,
) -> Result<(), ContractError> {
    TMP_GAS_BALANCE_AND_TIP_COST.update(deps.storage, |v| {
        if v.is_some() {
            Err(StdError::generic_err(
                "Too many purchase in queue! try later",
            ))
        } else {
            // find contract gas asset (uluna)
            let gas_asset = order.balance.gas.clone();

            let dca_contract_gas_balance = query_asset_balance(
                &deps.querier,
                env.contract.address.clone(),
                gas_asset.info.clone(),
            )?;

            Ok(Some((order.id(), dca_contract_gas_balance, tip_cost)))
        }
    })?;

    Ok(())
}

fn sanity_checks(
    env: &Env,
    contract_config: &Config,
    dca_order: &DcaInfo,
    hops: &Vec<SwapOperation>,
    tip_cost: Uint128,
) -> Result<(), ContractError> {
    // check balance deposit > dca_amount.amount
    if dca_order
        .balance
        .source
        .amount
        .lt(&dca_order.dca_amount.amount)
    {
        return Err(ContractError::InsufficientBalance {});
    }

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

    if first_hop != dca_order.balance.source.info {
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

fn update_balance(order: &mut DcaInfo, env: &Env, tip_cost: Asset) -> Result<(), ContractError> {
    // subtract dca_amount from order and update last_purchase time
    order.balance.source.amount = order
        .balance
        .source
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

    // Update balance target
    // This is done in the reply method below.

    order.balance.tip.amount = order
        .balance
        .tip
        .amount
        .checked_sub(tip_cost.amount)
        .map_err(|_| ContractError::InsufficientBalance {})?;

    order.balance.last_purchase = env.block.time.seconds();

    // Update gas balance
    // This is done in the reply method below.

    Ok(())
}

fn build_messages(
    env: &Env,
    info: &MessageInfo,
    contract_config: &Config,
    order: &mut DcaInfo,
    hops: Vec<SwapOperation>,
    max_spread: Decimal,
    tip_cost: Asset,
) -> Result<(Vec<SubMsg>, Vec<CosmosMsg>), ContractError> {
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut sub_messages: Vec<SubMsg> = vec![];
    // add funds and router message to response
    if let AssetInfo::Token { contract_addr } = &order.balance.source.info {
        // send a Transfer request to the token to the router
        messages.push(
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    // owner: user_address.to_string(),
                    recipient: contract_config.router_addr.to_string(),
                    amount: order.dca_amount.amount,
                })?,
            }
            .into(),
        );
    }

    // if it is a native token, we need to send the funds
    let funds = match &order.balance.source.info {
        AssetInfo::NativeToken { denom } => vec![Coin {
            amount: order.dca_amount.amount,
            denom: denom.clone(),
        }],
        AssetInfo::Token { .. } => vec![],
    };

    // tell the router to perform swap operations
    sub_messages.push(SubMsg {
        msg: WasmMsg::Execute {
            contract_addr: contract_config.router_addr.to_string(),
            funds,
            msg: to_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                operations: hops,
                minimum_receive: None,
                to: Some(env.contract.address.clone()), // In the end send the target asset back to the dca contract
                max_spread: Some(max_spread),
            })?,
        }
        .into(),
        id: PERFORM_DCA_PURCHASE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    // add tip payment to messages
    match &tip_cost.info {
        AssetInfo::NativeToken { denom } => messages.push(
            BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    amount: tip_cost.amount,
                    denom: denom.to_string(),
                }],
            }
            .into(),
        ),
        AssetInfo::Token { contract_addr } => messages.push(
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: tip_cost.amount,
                })?,
            }
            .into(),
        ),
    }

    Ok((sub_messages, messages))
}

/// # Description
/// The entry point to the contract for processing the reply from the submessage.
/// # Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`Reply`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let sub_msg_res = msg.clone().result.unwrap();
    let events = sub_msg_res.events;

    let gas_and_tip = TMP_GAS_BALANCE_AND_TIP_COST.load(deps.storage)?;

    match gas_and_tip.clone() {
        None => return Err(ContractError::TmpContractTargetBalance {}),
        Some((dca_order_id, contract_gas_before_purchase, tip_cost)) => {
            let mut order = DCA_ORDERS.load(deps.as_ref().storage, dca_order_id.clone())?;

            // Update target balance
            let target_amount_from_purchase =
                get_target_amount_from_purchase(&env, order.clone(), events)?;

            order.balance.target.amount = order.balance.target.amount + target_amount_from_purchase;

            // Update gas balance:

            let (
                order_gas_fee,
                contract_gas_before_purchase_2,
                tip_cost_2,
                target_amount_from_purchase_2,
                contract_gas_balance_after_purchase_2,
            ) = calculate_oder_gas_fee(
                &deps,
                &env,
                &order,
                contract_gas_before_purchase.clone(),
                tip_cost.clone(),
                target_amount_from_purchase,
            )?;
            order.balance.gas.amount = order.balance.gas.amount + order_gas_fee;

            DCA_ORDERS.save(deps.storage, dca_order_id.clone(), &order)?;
            TMP_GAS_BALANCE_AND_TIP_COST.save(deps.storage, &None)?;

            let mut data = msg.result.unwrap();

            let e = Event::new("my_event")
                .add_attribute("target_amount-purchase", target_amount_from_purchase)
                .add_attribute("order_gas_fee", order_gas_fee)
                .add_attribute(
                    "contract_gas_before_purchase_2",
                    contract_gas_before_purchase_2.to_string(),
                )
                .add_attribute("tip_cost_2", tip_cost_2.to_string())
                .add_attribute(
                    "target_amount_from_purchase_2",
                    target_amount_from_purchase_2.to_string(),
                )
                .add_attribute(
                    "contract_gas_balance_after_purchase_2",
                    contract_gas_balance_after_purchase_2.to_string(),
                )
                .add_attribute("tip_cost", tip_cost.to_string());
            data.events.push(e);

            SUB_MSG_DATA.save(deps.storage, &data)?;

            return Ok(Response::default());
        }
    }
    /*

    let data = msg.result.unwrap();
    TMP_CONTRACT_TARGET_GAS_BALANCE.save(deps.storage, &None)?;
    SUB_MSG_DATA.save(deps.storage, &data)?;
    Ok(Response::default())
    */
}

fn get_swap_events(
    //   deps: &DepsMut,
    //   order: DcaInfo,
    events: Vec<Event>,
) -> Result<Vec<Event>, ContractError> {
    let swap_events = events
        .into_iter()
        .filter(|event| {
            event.ty == "wasm"
                && event.attributes[1].key == "action"
                && event.attributes[1].value == "swap"
        })
        .collect::<Vec<Event>>();

    let l = swap_events.len();
    if l == 0 {
        return Err(ContractError::InvalidSwapOperations {
            msg: "The router didn't perform any swaps!".to_string(),
        });
    }
    // check_spread_threshold_condition(deps, order, swap_events.clone())?;

    Ok(swap_events)
}
fn check_spread_threshold_condition(
    deps: &DepsMut,
    order: DcaInfo,
    swap_events: Vec<Event>,
) -> Result<(), ContractError> {
    let mut max_spread_op = order.max_spread;
    if max_spread_op == None {
        let config = CONFIG.load(deps.storage)?;
        max_spread_op = Some(config.max_spread);
    }
    let max_spread = max_spread_op.unwrap();

    let swap_spreads = swap_events
        .into_iter()
        .map(|e| {
            (
                e.attributes
                    .clone()
                    .into_iter()
                    .find(|a| a.key == "offer_amount")
                    .clone()
                    .expect(&"no attribute 'offer_amount' in the swap event")
                    .value,
                e.attributes
                    .into_iter()
                    .find(|a| a.key == "spread_amount")
                    .expect(&"no attribute 'spread_amount' in the swap event")
                    .value,
            )
        })
        .map(|s| (Uint128::from_str(&s.0).ok(), Uint128::from_str(&s.1).ok()))
        .collect::<Vec<(Option<Uint128>, Option<Uint128>)>>();

    for (offer_amount_op, spread_amount_op) in swap_spreads {
        let offer_amount = offer_amount_op.ok_or(ContractError::InvalidInput {
            msg: format!("Could not parse offer_amount:{:?}", offer_amount_op),
        })?;

        let spread_amount = spread_amount_op.ok_or(ContractError::InvalidInput {
            msg: format!("Could not parse spread_amount:{:?}", spread_amount_op),
        })?;
        let swap_spread_ratio = Decimal::from_ratio(spread_amount, offer_amount);

        if swap_spread_ratio.gt(&max_spread) {
            return Err(ContractError::MaxSpreadCheckFail {
                max_spread: max_spread.to_string(),
                swap_spread: swap_spread_ratio.to_string(),
            });
        }
    }
    Ok(())
}

fn get_target_amount_from_purchase(
    env: &Env,
    order: DcaInfo,
    events: Vec<Event>,
) -> Result<Uint128, ContractError> {
    let swap_events = get_swap_events(events)?;
    let l = swap_events.len();
    let last_swap = &swap_events[l - 1];

    let ls_map = last_swap
        .attributes
        .clone()
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect::<Map<String, String>>();

    let ask_asset = match order.balance.target.info {
        AssetInfo::Token { contract_addr } => contract_addr.to_string(),
        AssetInfo::NativeToken { denom } => denom.to_string(),
    };

    let expected_data = vec![
        ("receiver", env.contract.address.to_string()),
        ("ask_asset", ask_asset),
    ];

    // sanity checks:
    for (key, value) in expected_data {
        match ls_map.get(key) {
            None => {
                return Err(ContractError::InvalidSwapOperations {
                    msg: format!("Attribute with key={:} is missing in the lst swap ", key),
                })
            }
            Some(map_value) => {
                if map_value != &value {
                    return Err(ContractError::InvalidSwapOperations {
                                msg: format!(
                                    "Invalid attribute in the last swap: (key={:}, value={:}). Expected value={:} ",
                                    key, map_value, value
                                ),
                            });
                }
            }
        }
    }

    let target_amount_op = ls_map.get("return_amount");

    match target_amount_op {
        None => {
            return Err(ContractError::InvalidSwapOperations {
                msg: format!("Attribute with key='return_amount' is missing in the last swap"),
            })
        }
        Some(target_amount) => return Ok(Uint128::from_str(target_amount)?),
    }
}

fn calculate_oder_gas_fee(
    deps: &DepsMut,
    env: &Env,
    order: &DcaInfo,
    contract_gas_before_purchase: Asset,
    tip_cost: Asset,
    target_amount_from_purchase: Uint128,
) -> Result<(Uint128, Uint128, Uint128, Uint128, Uint128), ContractError> {
    //  contract_gas_balance_before_purchase - (order gas fee) - (order tip_cost)? + (order target)? = contract_gas_balance_after_purchase
    //  => (order gas fee) = contract_gas_balance_before_purchase  - (order tip_cost)? + (order target)? - contract_gas_balance_after_purchase
    let contract_gas_balance_after_purchase = query_asset_balance(
        &deps.querier,
        env.contract.address.clone(), // dca contract
        order.balance.gas.info.clone(),
    )?;

    let mut temp_gas_balance = contract_gas_before_purchase.amount.clone();
    if &order.balance.gas.info == &tip_cost.info {
        temp_gas_balance = temp_gas_balance - tip_cost.amount
    }
    if &order.balance.gas.info == &order.balance.target.info {
        temp_gas_balance = temp_gas_balance + target_amount_from_purchase
    }

    let order_gas_fee = temp_gas_balance - contract_gas_balance_after_purchase.amount;
    Ok((
        order_gas_fee,
        contract_gas_before_purchase.amount,
        tip_cost.amount,
        target_amount_from_purchase,
        contract_gas_balance_after_purchase.amount,
    ))
}

#[cfg(test)]
mod tests {
    use astroport::{asset::AssetInfo, router::SwapOperation};
    use astroport_dca::dca::ExecuteMsg;
    use cosmwasm_std::{
        attr,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Coin, Empty, Response,
    };

    use crate::contract::execute;
    use crate::fixture::fixture::mock_storage_valid_data;

    #[test]
    fn test_perform_dca_purchase_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds: Vec<Coin> = vec![]; //[coin(100, "ibc/usdx")];
        let info = mock_info("creator", &funds);

        let dca_info_id = "2".to_string();

        // ibc/usdc -> asset0 --> target_2_addr
        let hops = vec![
            SwapOperation::AstroSwap {
                offer_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("ibc/usdc"),
                },
                ask_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0"),
                },
            },
            SwapOperation::AstroSwap {
                offer_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0"),
                },
                ask_asset_info: AssetInfo::NativeToken {
                    denom: ("asset1".to_string()),
                },
            },
            SwapOperation::AstroSwap {
                offer_asset_info: AssetInfo::NativeToken {
                    denom: ("asset1".to_string()),
                },
                ask_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("target_2_addr"),
                },
            },
        ];

        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::PerformDcaPurchase {
            dca_order_id: dca_info_id.clone(),
            hops: hops.clone(),
        };

        // execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "perform_dca_purchase"),
            attr("dca_order_id", "2"),
            attr("hops", format!("{:?}", hops)),
        ]);

        assert_eq!(actual_response.attributes, expected_response.attributes);
        //   assert_eq!(format!("{:?}", actual_response.messages), "[SubMsg { id: 0, msg: Wasm(Execute { contract_addr: \"tip_2_addr\", msg: Binary(7b227472616e736665725f66726f6d223a7b226f776e6572223a2263726561746f72222c22726563697069656e74223a22636f736d6f7332636f6e7472616374222c22616d6f756e74223a223530227d7d), funds: [] }), gas_limit: None, reply_on: Never }]")
    }
}
