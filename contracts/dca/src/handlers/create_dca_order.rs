use crate::state::{WhitelistTokens, CONFIG};
use crate::{error::ContractError, get_token_allowance::get_token_allowance, state::USER_DCA};
use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::{DcaInfo, PurchaseSchedule};
use cosmwasm_std::{attr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
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
    deposit_assets: Vec<Asset>,
    tip_assets: Vec<Asset>,
    target_asset: AssetInfo,
    gas: Asset,
    purchase_schedules: Vec<PurchaseSchedule>,
    max_hops: Option<u32>,
    max_spread: Option<Decimal>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_tokens = config.whitelist_tokens;

    let mut orders = USER_DCA
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    let created_at = env.block.time.seconds();
    let id = Uuid::new_v4().simple().to_string();

    // start_at > created_at
    // target_asset whitelisted and > amount >0
    // deposit_asset whitelisted and > amount > 0
    // tip_asset whitelisted and amount > 0
    // gas amount > 0

    sanity_checks(
        &deps,
        &env,
        &info,
        &whitelist_tokens,
        // interval,
        &deposit_assets,
        &tip_assets,
        //  target_asset.clone(),
        // gas.clone(),
        &purchase_schedules,
    )?;

    // store dca order
    orders.push(DcaInfo::new(
        id.clone(),
        env.block.time.seconds(),
        start_at,
        interval,
        target_asset.clone(),
        0,
        deposit_assets.clone(),
        tip_assets.clone(),
        gas.clone(),
        purchase_schedules.clone(),
        max_hops,
        max_spread,
    ));

    USER_DCA.save(deps.storage, &info.sender, &orders)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "create_dca_order"),
        attr("id", id),
        attr("created_at", created_at.to_string()),
        attr("start_at", start_at.to_string()),
        attr("interval", interval.to_string()),
        attr("deposit_assets", format!("{:?}", deposit_assets)),
        attr("tip_assets", format!("{:?}", tip_assets)),
        attr("target_asset", format!("{:?}", target_asset)),
        attr("purchase_schedules", format!("{:?}", purchase_schedules)),
        attr("gas", format!("{:?}", gas)),
        attr("max_hops", format!("{:?}", max_hops)),
        attr("max_spread", format!("{:?}", max_spread)),
    ]))
}

fn sanity_checks(
    deps: &DepsMut,
    env: &Env,
    info: &MessageInfo,
    whitelist_tokens: &WhitelistTokens,
    // interval: u64,
    deposit_assets: &Vec<Asset>,
    tip_assets: &Vec<Asset>,
    // target_asset: AssetInfo,
    //  gas: Asset,
    purchase_schedules: &Vec<PurchaseSchedule>,
) -> StdResult<()> {
    let asset_map = &mut HashMap::new();

    // check deposited assets list is not empty
    if deposit_assets.len() == 0 {
        return Err(StdError::generic_err("Deposit_assets list empty"));
    }

    let mut unique_asset_info_list: &mut Vec<AssetInfo> = &mut vec![];
    for asset in deposit_assets.iter() {
        // check asset is in the Whitelist
        if !whitelist_tokens.is_deposit_asset(&asset.info) {
            return Err(StdError::generic_err(format!(
                "Deposited asset, {:?},  not whitelisted",
                asset.info
            )));
        }

        // check no duplicated asset info in the deposit list
        unique_asset_info_check(&mut unique_asset_info_list, asset.info.clone())?;

        aggregate_assets(asset_map, asset.clone());
    }

    let unique_asset_info_list = &mut vec![];
    // check tip assets are whitelisted
    for asset in tip_assets.iter() {
        if !whitelist_tokens.is_tip_asset(&asset.info) {
            return Err(StdError::generic_err(format!(
                " tip asset, {:?},  not whitelisted",
                asset.info
            )));
        }

        // check no duplicated asset info in the tip list
        unique_asset_info_check(unique_asset_info_list, asset.info.clone())?;

        aggregate_assets(asset_map, asset.clone());
    }

    for (_, asset) in asset_map {
        asset.assert_sent_native_token_balance(&info)?;
        // check that user has sent the valid tokens to the contract
        // if native token, they should have included it in the message
        // otherwise, if cw20 token, they should have provided the correct allowance
        match &asset.info {
            AssetInfo::NativeToken { .. } => asset.assert_sent_native_token_balance(&info)?,
            AssetInfo::Token { contract_addr } => {
                let allowance =
                    get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
                if allowance != asset.amount {
                    return Err(StdError::generic_err(format!(
                        "Aggregated asset amount in the DCA order {:?} is not equal to allowance set by token",
                        asset
                    )));
                }
            }
        }
    }

    // Check purchase_schedule consistent with the deposited assets
    if deposit_assets.len() != purchase_schedules.len() {
        return Err(StdError::generic_err(
            "Not all deposit assets have a corresponding purchase schedule or viceversa",
        ));
    }

    /*
    let deposit_asset_info_list = deposit_assets
        .iter()
        .map(|a| a.info.clone())
        .collect::<Vec<AssetInfo>>();

        */
    let unique_asset_info_list = &mut vec![];
    for ps in purchase_schedules.iter() {
        /*
                if !deposit_asset_info_list.contains(&ps.asset_info) {
                    return Err(StdError::generic_err(format!(
                        "Purchase schedule invalid. The asset, {:?}, does not appear in the deposit asset list",
                        ps.asset_info
                    )));
                }
        */

        // this flag is true if purchase asset info is included in the deposit asset info
        let mut included_pc_asset_info = false;
        for da in deposit_assets.iter() {
            if da.info == ps.asset_info {
                included_pc_asset_info = true;

                // check that purchase asset amount is less than deposited asset amount
                if ps.amount.gt(&da.amount) {
                    return Err(StdError::generic_err(format!(
                        "Deposited asset, {:?},  is not divisible by the purchase amount, {:?}",
                        da, ps
                    )));
                }

                /*
                // This check is maybe to restrictive?
                if !da
                    .amount
                    .checked_rem(ps.amount)
                    .map_err(|e| StdError::DivideByZero { source: e })?
                    .is_zero()
                {
                    return Err(StdError::generic_err(format!(
                        "Deposited asset, {:?},  is not divisible by the purchase amount, {:?}",
                        da, ps
                    )));
                }

                */
            }
        }

        if included_pc_asset_info == false {
            return Err(StdError::generic_err(format!(
                "Purchase schedule invalid. The asset, {:?}, does not appear in the deposit asset list",
                ps.asset_info
            )));
        }

        // check no duplicated asset info in the purchase schedule list
        unique_asset_info_check(unique_asset_info_list, ps.asset_info.clone())?;
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

fn unique_asset_info_check(
    asset_info_list: &mut Vec<AssetInfo>,
    asset_info: AssetInfo,
) -> StdResult<()> {
    if asset_info_list.contains(&asset_info) {
        return Err(StdError::generic_err(format!(
            "Duplicate asset info, {:?}, in the tip asset list",
            asset_info
        )));
    } else {
        asset_info_list.push(asset_info.clone())
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use crate::state::{Config, WhitelistTokens, CONFIG, USER_DCA};
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::{ExecuteMsg, PurchaseSchedule};

    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info, MockStorage},
        Addr, MemoryStorage, Response, Uint128,
    };

    use crate::contract::execute;

    fn mock_storage() -> MemoryStorage {
        let mock_config = Config {
            whitelist_tokens: WhitelistTokens {
                deposit: vec![
                    AssetInfo::NativeToken {
                        denom: "usdt".to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: "uluna".to_string(),
                    },
                    AssetInfo::Token {
                        contract_addr: Addr::unchecked("asset0"),
                    },
                ],
                tip: vec![
                    AssetInfo::NativeToken {
                        denom: "usdt".to_string(),
                    },
                    AssetInfo::Token {
                        contract_addr: Addr::unchecked("asset1"),
                    },
                ],
            },
            factory_addr: Addr::unchecked("XXX"),
            router_addr: Addr::unchecked("YYY"),
        };

        let mut store = MockStorage::new();

        _ = CONFIG.save(&mut store, &mock_config);

        return store;
    }

    #[test]
    // deposit assets are whitelisted
    fn test_create_dca_order_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage();

        let funds = [coin(15, "usdt"), coin(100, "uluna")];
        let info = mock_info("creator", &funds);

        // build msg
        let deposit_assets = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(10u128),
        }];

        let tip_assets = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(5u128),
        }];

        let target_asset = AssetInfo::Token {
            contract_addr: Addr::unchecked("XXX"),
        };

        let gas = Asset {
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
            amount: Uint128::from(100u128),
        };

        let purchase_schedules = vec![PurchaseSchedule {
            asset_info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(10u128),
            interval: 100u64,
        }];
        let msg = ExecuteMsg::CreateDcaOrder {
            start_at: 100u64,
            interval: 100u64,
            deposit_assets: deposit_assets.clone(),
            tip_assets: tip_assets.clone(),
            target_asset: target_asset.clone(),
            gas: gas.clone(),
            purchase_schedules: purchase_schedules.clone(),
            max_hops: None,
            max_spread: None,
        };

        // execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let orders = USER_DCA
            .load(deps.as_ref().storage, &info.sender)
            .unwrap_or_default();

        assert_eq!(1, orders.len());

        // let dca_0 {iy}= orders[0].clone();
        let expected_response = Response::new().add_attributes(vec![
            attr("action", "create_dca_order"),
            attr("id", orders[0].to_owned().id()),
            attr("created_at", orders[0].to_owned().created_at().to_string()),
            attr("start_at", orders[0].start_at.to_string()),
            attr("interval", orders[0].interval.to_string()),
            attr("deposit_assets", format!("{:?}", orders[0].deposit_assets)),
            attr("tip_assets", format!("{:?}", orders[0].tip_assets)),
            attr("target_asset", format!("{:?}", orders[0].target_asset)),
            attr(
                "purchase_schedules",
                format!("{:?}", orders[0].purchase_schedules),
            ),
            attr("gas", format!("{:?}", orders[0].gas)),
            attr("max_hops", format!("{:?}", orders[0].max_hops)),
            attr("max_spread", format!("{:?}", orders[0].max_spread)),
        ]);

        assert_eq!(actual_response, expected_response);
    }
}
