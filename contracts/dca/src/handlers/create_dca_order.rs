use crate::state::{Config, WhitelistTokens, CONFIG};
use crate::{
    error::ContractError, get_token_allowance::get_token_allowance, state::USER_DCA_ORDERS,
};
use astroport::asset::{Asset, AssetInfo, ULUNA_DENOM};
use astroport_dca::dca::{Balance, DcaInfo};
use cosmwasm_std::{
    attr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use std::collections::HashMap;
use uuid::Uuid;

/// ## Description
/// Creates a new DCA order for a user where the `target_asset` will be purchased with `dca_amount`
/// of token `initial_asset` every `interval`.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to create their order, containing the
/// [`AssetInfo::NativeToken`] if the `initial_asset` is a native token.
///
/// * `initial_asset` - The [`Asset`] that is being spent to purchase DCA orders. If the asset is a
/// Token (non-native), the contact will need to have the allowance for the DCA contract set to the
/// `initial_asset.amount`.
///
/// * `target_asset` - The [`AssetInfo`] that is being purchased with `initial_asset`.
///
/// * `interval` - The time in seconds between DCA purchases.
///
/// * `dca_amount` - A [`Uint128`] representing the amount of `initial_asset` to spend each DCA
/// purchase.
pub fn create_dca_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_at: u64,
    interval: u64,
    dca_amount: Asset,
    max_hops: Option<u32>,
    max_spread: Option<Decimal>,
    deposit: Asset,
    tip: Asset,
    gas: Asset,
    target_info: AssetInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_tokens = config.whitelist_tokens.clone();

    let mut orders = USER_DCA_ORDERS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    let created_at = env.block.time.seconds();
    let id = Uuid::new_v4().simple().to_string();

    // start_at > created_at
    // target_asset whitelisted and  amount >0
    // deposit_asset whitelisted and  amount > 0
    // tip_asset whitelisted and amount > 0
    // gas amount > 0
    sanity_checks(
        &deps,
        &env,
        &info,
        &config,
        &whitelist_tokens,
        &dca_amount,
        &deposit,
        &tip,
        &gas,
    )?;

    let balance = Balance {
        deposit: deposit.clone(),
        spent: Asset {
            info: deposit.info.clone(),
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
    orders.push(DcaInfo::new(
        id.clone(),
        env.block.time.seconds(),
        start_at,
        interval,
        dca_amount.clone(),
        max_hops,
        max_spread,
        balance.clone(),
    ));

    USER_DCA_ORDERS.save(deps.storage, &info.sender, &orders)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "create_dca_order"),
        attr("id", id),
        attr("created_at", created_at.to_string()),
        attr("start_at", start_at.to_string()),
        attr("interval", interval.to_string()),
        attr("dca_amount", dca_amount.to_string()),
        attr("max_hops", format!("{:?}", max_hops)),
        attr("max_spread", format!("{:?}", max_spread)),
        attr("deposit", format!("{:?}", deposit)),
        attr("tip", format!("{:?}", tip)),
        attr("gas", format!("{:?}", gas)),
        attr("target_info", format!("{:?}", target_info)),
    ]))
}

fn sanity_checks(
    deps: &DepsMut,
    env: &Env,
    info: &MessageInfo,
    config: &Config,
    whitelist_tokens: &WhitelistTokens,
    dca_amount: &Asset,
    deposit: &Asset,
    tip: &Asset,
    gas: &Asset,
) -> Result<(), ContractError> {
    let asset_map = &mut HashMap::new();

    // Check amount to spend at each purchase is of the same type of
    // deposit asset
    if !(dca_amount.info == deposit.info) {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "The asset type of dac_amount asset and deposit asset must be the same.
                 Got dac_amount asset type: {:?}  , deposit asset type: {:?}",
                dca_amount, deposit.info
            ),
        });
    }

    // check deposit asset is in the Whitelist
    if !whitelist_tokens.is_deposit_asset(&deposit.info) {
        return Err(ContractError::InvalidInput {
            msg: format!("Deposited asset, {:?},  not whitelisted", deposit.info),
        });
    }

    // check tip asset is whitelisted
    if !whitelist_tokens.is_tip_asset(&tip.info) {
        return Err(ContractError::InvalidInput {
            msg: format!(" tip asset, {:?},  not whitelisted", tip.info),
        });
    }

    // Check deposit asset amount > 0.
    // For Uint128 amount, it is equivalent to check that the amount is not zero.
    if deposit.amount.is_zero() {
        return Err(ContractError::InvalidInput {
            msg: format!("Expected Deposited asset > 0, got 0"),
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

    aggregate_assets(asset_map, deposit.clone());
    aggregate_assets(asset_map, tip.clone());
    aggregate_assets(asset_map, gas.clone());

    // let assets = vec![deposit, tip, gas];
    // aggregate_assets(asset_map, asset.clone());
    for (_, asset) in asset_map {
        // check that user has sent the valid tokens to the contract
        // if native token, they should have included it in the message
        // otherwise, if cw20 token, they should have provided the correct allowance
        match &asset.info {
            AssetInfo::NativeToken { .. } => asset.assert_sent_native_token_balance(&info)?,
            AssetInfo::Token { contract_addr } => {
                let allowance =
                    get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
                if allowance != asset.amount {
                    return Err(ContractError::InvalidTokenDeposit {
                        token: contract_addr.to_string(),
                    });
                }
            }
        }
    }

    return Ok(());
}

fn aggregate_assets(asset_map: &mut HashMap<String, Asset>, asset: Asset) {
    // if asset.info.is_native_token() {
    let key = asset.info.to_string();
    let op = asset_map.get(&key);
    match op {
        None => {
            asset_map.insert(key.clone(), asset.clone());
        }
        Some(a) => {
            let aggregated_amount = asset.amount.checked_add(a.amount).unwrap();
            let aggregated_asset = Asset {
                info: asset.info.clone(),
                amount: aggregated_amount,
            };

            asset_map.insert(key.clone(), aggregated_asset);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::USER_DCA_ORDERS;
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::{Balance, ExecuteMsg};

    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response, Uint128,
    };

    // use super::super::add_bot_tip::test_util::mock_config;

    use super::super::add_bot_tip::test_util::mock_storage_valid_data;
    use crate::contract::execute;

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
        let deposit = Asset {
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
            deposit: deposit.clone(),
            tip: tip.clone(),
            gas: gas.clone(),
            target_info: target_info.clone(),
        };

        // Check there are 2 DCA orders before executing the msg
        let mut orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap_or_default();

        assert_eq!(2, orders.len());

        // Execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Check there are 3 DCA orders after executing the msg
        orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap_or_default();

        assert_eq!(3, orders.len());

        // Check expected and actual response are the same
        let expected_response = Response::new().add_attributes(vec![
            attr("action", "create_dca_order"),
            attr("id", orders[2].id()),
            attr("created_at", orders[2].created_at().to_string()),
            attr("start_at", orders[2].start_at.to_string()),
            attr("interval", orders[2].interval.to_string()),
            attr("dca_amount", orders[2].dca_amount.to_string()),
            attr("max_hops", format!("{:?}", orders[2].max_hops)),
            attr("max_spread", format!("{:?}", orders[2].max_spread)),
            attr("deposit", format!("{:?}", orders[2].balance.deposit)),
            attr("tip", format!("{:?}", orders[2].balance.tip)),
            attr("gas", format!("{:?}", orders[2].balance.gas)),
            attr(
                "target_info",
                format!("{:?}", orders[2].balance.target.info),
            ),
        ]);

        assert_eq!(actual_response, expected_response);
    }
}
