use astroport::{
    asset::AssetInfo,
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{Config, CONFIG, DCA_ORDERS},
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
    dca_order_id: String,
    hops: Vec<SwapOperation>,
) -> Result<Response, ContractError> {
    let contract_config = CONFIG.load(deps.storage)?;
    let order = DCA_ORDERS.load(deps.as_ref().storage, dca_order_id.clone())?;
    let hops_len = hops.len() as u32;
    let tip_cost = contract_config
        .per_hop_fee
        .checked_mul(Uint128::from(hops_len))?;

    sanity_checks(&env, &contract_config, &order, &hops, tip_cost.clone())?;

    // store messages to send in response
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // load dca order and update it
    DCA_ORDERS.update(
        deps.storage,
        dca_order_id.clone(),
        |order| -> Result<DcaInfo, ContractError> {
            let order = &mut order.unwrap();

            // retrieve max_spread from user config, or default to contract set max_spread
            let max_spread = order.max_spread.unwrap_or(contract_config.max_spread);

            messages = build_messages(
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

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "perform_dca_purchase"),
        attr("dca_order_id", dca_order_id),
        attr("hops", format!("{:?}", hops)),
    ]))
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

    // todo: update order.balance.target
    // This will required to be able to read the response msg from the swap operation
    // Most likely it is possible using SubMsg

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
    contract_config: &Config,
    order: &mut DcaInfo,
    hops: Vec<SwapOperation>,
    max_spread: Decimal,
    tip_cost: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let user_address = order.created_by();
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
    messages.push(
        WasmMsg::Execute {
            contract_addr: contract_config.router_addr.to_string(),
            funds,
            msg: to_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                operations: hops,
                minimum_receive: None,
                to: Some(user_address.clone()), // todo: send the target asset about back to the DCA contract rather than to the user. and update balance.target accordingly.
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

    Ok(messages)
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
