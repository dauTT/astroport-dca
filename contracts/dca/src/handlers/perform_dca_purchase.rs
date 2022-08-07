// use std::clone;

use crate::utils::{query_asset_balance, try_sub};
use astroport::{
    asset::AssetInfo,
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    Reply, ReplyOn, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{Config, CONFIG, DCA_ORDERS, TMP_CONTRACT_TARGET_BALANCE},
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

    // Store the target asset balance of the dca contract before the
    // execution of the purchase
    store_dca_contract_target_balance(&mut deps, env.clone(), order.clone())?;
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
                tip_cost.clone(),
            )?;

            update_balance(order, &env, tip_cost.clone())?;

            return Ok(order.clone());
        },
    )?;

    Ok(Response::new()
        .add_submessages(sub_messages)
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "perform_dca_purchase"),
            attr("dca_order_id", dca_order_id),
            attr("hops", format!("{:?}", hops)),
        ]))
}

pub fn store_dca_contract_target_balance(
    deps: &mut DepsMut,
    env: Env,
    order: DcaInfo,
) -> Result<(), ContractError> {
    TMP_CONTRACT_TARGET_BALANCE.update(deps.storage, |v| {
        if v.is_some() {
            Err(StdError::generic_err(
                "Too many purchase in queue! try later",
            ))
        } else {
            // execute query to target asset
            let target_asset = order.balance.target.clone();

            let dca_contract_target_balance = query_asset_balance(
                &deps.querier,
                env.contract.address.clone(),
                target_asset.info.clone(),
            )?;

            Ok(Some((order.id(), dca_contract_target_balance)))
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

fn update_balance(order: &mut DcaInfo, env: &Env, tip_cost: Uint128) -> Result<(), ContractError> {
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
        .checked_sub(tip_cost)
        .map_err(|_| ContractError::InsufficientBalance {})?;

    order.balance.last_purchase = env.block.time.seconds();

    Ok(())
}

fn build_messages(
    env: &Env,
    info: &MessageInfo,
    contract_config: &Config,
    order: &mut DcaInfo,
    hops: Vec<SwapOperation>,
    max_spread: Decimal,
    tip_cost: Uint128,
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
        AssetInfo::Token { contract_addr } => messages.push(
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: tip_cost,
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
pub fn reply(deps: DepsMut, env: Env, _msg: Reply) -> Result<Response, ContractError> {
    let contract_target_balance_before_purchase = TMP_CONTRACT_TARGET_BALANCE.load(deps.storage)?;

    match contract_target_balance_before_purchase.clone() {
        None => return Err(ContractError::TmpContractTargetBalance {}),
        Some((dca_order_id, target_balance_before_purchase)) => {
            let mut order = DCA_ORDERS.load(deps.as_ref().storage, dca_order_id.clone())?;

            let target_balance_after_purchase = query_asset_balance(
                &deps.querier,
                env.contract.address.clone(),
                order.balance.target.info.clone(),
            )?;
            let diff_amount = try_sub(
                target_balance_after_purchase,
                target_balance_before_purchase,
            )?;

            order.balance.target.amount = order.balance.target.amount + diff_amount.amount;
            DCA_ORDERS.save(deps.storage, dca_order_id, &order)?;

            return Ok(Response::default());
        }
    };
}

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

        let dca_info_id = "order_2".to_string();

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
            attr("dca_order_id", "order_2"),
            attr("hops", format!("{:?}", hops)),
        ]);

        assert_eq!(actual_response.attributes, expected_response.attributes);
        //   assert_eq!(format!("{:?}", actual_response.messages), "[SubMsg { id: 0, msg: Wasm(Execute { contract_addr: \"tip_2_addr\", msg: Binary(7b227472616e736665725f66726f6d223a7b226f776e6572223a2263726561746f72222c22726563697069656e74223a22636f736d6f7332636f6e7472616374222c22616d6f756e74223a223530227d7d), funds: [] }), gas_limit: None, reply_on: Never }]")
    }
}
