use crate::state::{Config, CONFIG};
use crate::{
    error::ContractError,
    state::{DCA_ORDERS, LAST_DCA_ORDER_ID, USER_DCA_ORDERS},
    utils::{aggregate_assets, validate_all_deposit_assets},
};
use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::{Balance, DcaInfo, WhitelistedTokens};
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use std::collections::HashMap;
use std::str::FromStr;

/// ## Description
/// Creates a new DCA order for a user where the `target_info` asset will be purchased with `dca_amount`
/// of token `source` asset at every `interval`.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to create their order, containing the
/// [`AssetInfo::NativeToken`] if the `source` asset is a native token.
///
/// * `start_at` - The [`u64`] the time in seconds defining the start of the dca order.
///
/// * `interval` - The time in seconds between DCA purchases.
///
///
/// * `dca_amount` - A [`Uint128`] representing the amount of `initial_asset` to spend each DCA
/// purchase.
///
/// * `max_hops` - An optional [`u32`] representing the maximum number of swap operation allowed to purchase the target asset.
///
/// * `max_spread` - An optional [`Decimal`] representing the maximum spread ratio allowed beween the swap operations.
///

/// * `source` - A whitelisted [`Asset`] that is being spent to purchase DCA orders. If the asset is a
/// Token (non-native), the contract will need to have the allowance for the DCA contract set to the
/// `source.amount`.
///
/// * `tip` - A whitelisted [`Asset`] which is used to reward a bot for performing purchases on behalf of the user.
///
/// * `gas` - the [`Asset`] (uluna) needed for the dca contract to performs transactions.
///
/// * `target_info` - The [`AssetInfo`] that is being purchased with `source` asset.
pub fn create_dca_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_at: u64,
    interval: u64,
    dca_amount: Asset,
    max_hops: Option<u32>,
    max_spread: Option<Decimal>,
    source: Asset,
    tip: Asset,
    gas: Asset,
    target_info: AssetInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelisted_tokens = config.whitelisted_tokens.clone();
    let mut user_dca_orders = USER_DCA_ORDERS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
    let created_at = env.block.time.seconds();

    // get the next unique id to use:
    let last_order_id = LAST_DCA_ORDER_ID
        .may_load(deps.storage)?
        .unwrap_or("0".to_string());

    let next_dca_order_id = Uint128::from_str(&last_order_id)?
        .checked_add(Uint128::from(1u128))?
        .to_string();

    // start_at > created_at
    // target_asset whitelisted and  amount >0
    // deposit_asset whitelisted and  amount > 0
    // tip_asset whitelisted and amount > 0
    // gas amount > 0
    // ...
    sanity_checks(
        &next_dca_order_id,
        &deps,
        &env,
        &info,
        &config,
        &whitelisted_tokens,
        &dca_amount,
        &source,
        &tip,
        &gas,
    )?;

    let balance = Balance {
        source: source.clone(),
        spent: Asset {
            info: source.info.clone(),
            amount: Uint128::zero(),
        },
        target: Asset {
            info: target_info.clone(),
            amount: Uint128::zero(),
        },
        tip: tip.clone(),
        gas: gas.clone(),
        last_purchase: 0u64,
    };
    // store dca order
    let dca = DcaInfo::new(
        next_dca_order_id.clone(),
        info.sender.clone(),
        env.block.time.seconds(),
        start_at,
        interval,
        dca_amount.clone(),
        max_hops,
        max_spread,
        balance.clone(),
    );
    user_dca_orders.push(next_dca_order_id.clone());

    DCA_ORDERS.save(deps.storage, next_dca_order_id.clone(), &dca)?;
    USER_DCA_ORDERS.save(deps.storage, &info.sender, &user_dca_orders)?;
    LAST_DCA_ORDER_ID.save(deps.storage, &next_dca_order_id)?;

    let mut messages: Vec<CosmosMsg> = vec![];

    for asset in vec![balance.source, balance.tip, balance.gas] {
        // The dca contract will only perform a TransferFrom from  token asset
        // Native assets needs already been sent before.
        if let AssetInfo::Token { contract_addr } = asset.info.clone() {
            messages.push(
                WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    funds: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: info.sender.to_string(),
                        recipient: env.contract.address.to_string().clone(),
                        amount: asset.amount,
                    })?,
                }
                .into(),
            );
        }
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "create_dca_order"),
        attr("id", next_dca_order_id),
        attr("created_at", created_at.to_string()),
        attr("start_at", start_at.to_string()),
        attr("interval", interval.to_string()),
        attr("dca_amount", dca_amount.to_string()),
        attr("max_hops", format!("{:?}", max_hops)),
        attr("max_spread", format!("{:?}", max_spread)),
        attr("source", format!("{:?}", source)),
        attr("tip", format!("{:?}", tip)),
        attr("gas", format!("{:?}", gas)),
        attr("target_info", format!("{:?}", target_info)),
    ]))
}

fn sanity_checks(
    next_dca_order_id: &String,
    deps: &DepsMut,
    env: &Env,
    info: &MessageInfo,
    config: &Config,
    whitelisted_tokens: &WhitelistedTokens,
    dca_amount: &Asset,
    source: &Asset,
    tip: &Asset,
    gas: &Asset,
) -> Result<(), ContractError> {
    // Check next_dca_order_id is not already used
    let res = DCA_ORDERS.load(deps.storage, next_dca_order_id.clone());
    if let Ok(_) = res {
        return Err(ContractError::DCAUniqueContraintViolation {
            id: next_dca_order_id.clone(),
        });
    }

    // Check amount to spend at each purchase is of the same type of
    // deposit asset
    if !(dca_amount.info == source.info) {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "The asset type of dac_amount asset and source asset must be the same.
                 Got dac_amount asset type: {:?}  , source asset type: {:?}",
                dca_amount, source.info
            ),
        });
    }

    // check source asset is in the Whitelist
    if !whitelisted_tokens.is_source_asset(&source.info) {
        return Err(ContractError::InvalidInput {
            msg: format!("Source asset, {:?},  not whitelisted", source.info),
        });
    }

    // check tip asset is whitelisted
    if !whitelisted_tokens.is_tip_asset(&tip.info) {
        return Err(ContractError::InvalidInput {
            msg: format!(" tip asset, {:?},  not whitelisted", tip.info),
        });
    }

    // Check source asset amount > 0.
    // For Uint128 amount, it is equivalent to check that the amount is not zero.
    if source.amount.is_zero() {
        return Err(ContractError::InvalidInput {
            msg: format!("Expected source asset > 0, got 0"),
        });
    }

    // Check tip amount > 0
    // For Uint128 amount, it is equivalent to check that the amount is not zero.
    if tip.amount.is_zero() {
        return Err(ContractError::InvalidInput {
            msg: format!("Expected tip asset > 0, got 0"),
        });
    }

    // Check gas amount > 0
    // For Uint128 amount, it is equivalent to check that the amount is not zero.
    if gas.amount.is_zero() {
        return Err(ContractError::InvalidInput {
            msg: format!("Expected gas amount  > 0, got 0"),
        });
    }

    // check gas is the native gas asset of the chain
    if !(gas.info == config.gas_info) {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "Expected gas to be {}, got {:?} ",
                config.gas_info,
                gas.info.clone()
            ),
        });
    }

    let asset_map = &mut HashMap::new();
    aggregate_assets(asset_map, source.clone());
    aggregate_assets(asset_map, tip.clone());
    aggregate_assets(asset_map, gas.clone());
    validate_all_deposit_assets(deps, &env, &info, asset_map)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::contract::execute;
    use crate::fixture::fixture::mock_storage_valid_data;
    use crate::state::{DCA_ORDERS, USER_DCA_ORDERS};
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::ExecuteMsg;
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response, Uint128,
    };

    #[test]
    // deposit assets are whitelisted
    fn test_create_dca_order_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("creator", &funds);

        // build msg
        let dca_amount = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(5u128),
        };
        let source = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(100u128),
        };
        let tip = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(100u128),
        };
        let gas = Asset {
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
            amount: Uint128::from(100u128),
        };
        let target_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract_addr"),
        };

        let msg = ExecuteMsg::CreateDcaOrder {
            start_at: 100u64,
            interval: 100u64,
            dca_amount: dca_amount.clone(),
            max_hops: None,
            max_spread: None,
            source: source.clone(),
            tip: tip.clone(),
            gas: gas.clone(),
            target_info: target_info.clone(),
        };

        // Check there are 2 DCA orders before executing the msg
        let mut user_orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap();

        assert_eq!(user_orders.len(), 2);

        /*
                let mut orders = USER_DCA_ORDERS
                    .load(deps.as_ref().storage, &info.sender)
                    .unwrap_or_default();
        */

        // Execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Check there are 3 DCA orders after executing the msg
        user_orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap();

        assert_eq!(user_orders.len(), 3);

        let order_id = user_orders[2].clone();
        assert_eq!(order_id, "3");

        let order = DCA_ORDERS
            .load(deps.as_ref().storage, order_id.clone())
            .unwrap();

        // Check expected and actual response are the same

        let expected_response = Response::new().add_attributes(vec![
            attr("action", "create_dca_order"),
            attr("id", order.id()),
            attr("created_at", order.created_at().to_string()),
            attr("start_at", order.start_at.to_string()),
            attr("interval", order.interval.to_string()),
            attr("dca_amount", order.dca_amount.to_string()),
            attr("max_hops", format!("{:?}", order.max_hops)),
            attr("max_spread", format!("{:?}", order.max_spread)),
            attr("source", format!("{:?}", order.balance.source)),
            attr("tip", format!("{:?}", order.balance.tip)),
            attr("gas", format!("{:?}", order.balance.gas)),
            attr("target_info", format!("{:?}", order.balance.target.info)),
        ]);

        assert_eq!(actual_response, expected_response);
    }
}
