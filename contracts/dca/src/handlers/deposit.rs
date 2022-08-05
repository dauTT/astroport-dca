use crate::{error::ContractError, state::DCA_ORDERS};
use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::{find_asset_info, DcaAssetType, DcaInfo};
use cosmwasm_std::{attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg};
use cw20::Cw20ExecuteMsg;

/// ## Description
/// Deposit assets (source, tip, gas) into the DCA contract
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `asset` - Asset [`DepsMut`] that contains the dependencies.
pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deposit_type: DcaAssetType,
    dca_order_id: String,
    asset: Asset,
) -> Result<Response, ContractError> {
    let order = DCA_ORDERS.load(deps.storage, dca_order_id.clone())?;

    // permission check
    if info.sender != order.created_by() {
        return Err(ContractError::Unauthorized {});
    }

    DCA_ORDERS.update(
        deps.storage,
        dca_order_id.clone(),
        |order| -> Result<DcaInfo, ContractError> {
            let dca_order = &mut order.unwrap();
            let new_order = try_add(dca_order, deposit_type.clone(), asset.clone())?;
            Ok(new_order.clone())
        },
    )?;

    // For native token we check if they have been sent.
    // (For token contract, the DCA contract will execute TransferFrom)
    asset.assert_sent_native_token_balance(&info)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let AssetInfo::Token { contract_addr, .. } = &asset.info {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: asset.amount,
            })?,
            funds: vec![],
        }));
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "deposit"),
        attr("dca_order_id", dca_order_id),
        attr("deposit_type", format!("{:?}", deposit_type)),
        attr("asset", format!("{:?}", asset)),
    ]))
}

pub fn try_add(
    order: &mut DcaInfo,
    asset_type: DcaAssetType,
    asset: Asset,
) -> Result<&mut DcaInfo, ContractError> {
    let order_asset_info = find_asset_info(asset_type.clone(), order.clone());

    if asset.info != order_asset_info {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "Expetected {:?} asset info:{:?}.  Got: {:?}",
                asset_type, order_asset_info, asset.info
            ),
        });
    };

    match asset_type {
        DcaAssetType::Source => {
            order.balance.source.amount = order.balance.source.amount.checked_add(asset.amount)?
        }
        DcaAssetType::Spent => {
            return Err(ContractError::InvalidInput {
                msg: format!("asset_type '{:?}' is not allowed", DcaAssetType::Spent),
            })
        }

        DcaAssetType::Target => {
            return Err(ContractError::InvalidInput {
                msg: format!("asset_type '{:?}' is not allowed", DcaAssetType::Target),
            })
        }
        DcaAssetType::Tip => {
            order.balance.tip.amount = order.balance.tip.amount.checked_add(asset.amount)?
        }
        DcaAssetType::Gas => {
            order.balance.gas.amount = order.balance.gas.amount.checked_add(asset.amount)?
        }
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::ExecuteMsg;

    use super::super::deposit::test_util::mock_storage_valid_data;
    use crate::{contract::execute, state::DCA_ORDERS};
    use astroport_dca::dca::DcaAssetType;
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, Response, Uint128,
    };

    #[test]
    fn test_deposit_source_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);

        // build msg
        let deposit_type = DcaAssetType::Source;
        let dac_order_id = "order_1".to_string();
        let asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(100u128),
        };

        let msg = ExecuteMsg::Deposit {
            deposit_type: deposit_type.clone(),
            dca_order_id: "order_1".to_string(),
            asset: asset.clone(),
        };

        //Check  amount before execution
        let mut order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.clone())
            .unwrap();

        assert_eq!(order.balance.source.amount, Uint128::from(100u128));

        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        //Check deposit amount after execution
        order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.clone())
            .unwrap();

        assert_eq!(order.balance.source.amount, Uint128::from(200u128));

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "deposit"),
            attr("dca_order_id", dac_order_id),
            attr("deposit_type", format!("{:?}", DcaAssetType::Source)),
            attr("asset", format!("{:?}", asset)),
        ]);
        assert_eq!(actual_response.attributes, expected_response.attributes);
    }

    #[test]
    fn test_deposit_token_unsupported() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "order_1";

        // build msg
        let deposit_type = DcaAssetType::Source;
        let asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "xxx".to_string(),
            },
            amount: Uint128::from(100u128),
        };

        let msg = ExecuteMsg::Deposit {
            deposit_type: deposit_type.clone(),
            dca_order_id: dac_order_id.to_string(),
            asset: asset.clone(),
        };

        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(actual_response.is_err(), true);
        let err = actual_response.err().unwrap();
        assert_eq!(
            err.to_string(),
            "Invalid input. msg: 'Expetected Source asset info:NativeToken { denom: \"usdt\" }.  Got: NativeToken { denom: \"xxx\" }'"
        );
    }

    #[test]
    fn test_deposit_native_token_amount_mismatch() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "order_1".to_string();

        // build msg
        let deposit_type = DcaAssetType::Source;
        let asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(1u128),
        };
        let msg = ExecuteMsg::Deposit {
            deposit_type: deposit_type.clone(),
            dca_order_id: dac_order_id,
            asset: asset.clone(),
        };
        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(actual_response.is_err(), true);
        let err = actual_response.err().unwrap();
        assert_eq!(
            err.to_string(),
            "Generic error: Native token balance mismatch between the argument and the transferred"
        );
    }

    #[test]
    fn test_deposit_tip_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "ibc/usdx"), coin(50, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "order_1".to_string();

        // build msg
        let deposit_type = DcaAssetType::Tip;
        let tip_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(50u128),
        };

        //Check  amount before execution
        let mut order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.clone())
            .unwrap();
        assert_eq!(order.balance.tip.amount, Uint128::from(5u128));

        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::Deposit {
            deposit_type: deposit_type.clone(),
            dca_order_id: dac_order_id.clone(),
            asset: tip_asset.clone(),
        };
        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        //Check deposit amount after execution
        order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.clone())
            .unwrap();
        assert_eq!(order.balance.tip.amount, Uint128::from(55u128));

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "deposit"),
            attr("dca_order_id", "order_1"),
            attr("deposit_type", format!("{:?}", DcaAssetType::Tip)),
            attr("asset", format!("{:?}", tip_asset)),
        ]);
        assert_eq!(actual_response.attributes, expected_response.attributes);
    }

    #[test]
    fn test_deposit_target_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "ibc/usdx"), coin(50, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "order_1".to_string();

        // build msg
        let deposit_type = DcaAssetType::Target;
        let target_asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("target_1_addr"),
            },
            amount: Uint128::from(50u128),
        };

        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::Deposit {
            deposit_type: deposit_type.clone(),
            dca_order_id: dac_order_id.clone(),
            asset: target_asset.clone(),
        };
        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert_eq!(
            actual_response.to_string(),
            "Invalid input. msg: 'asset_type 'Target' is not allowed'"
        )
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::state::{Config, WhitelistTokens, CONFIG, DCA_ORDERS, USER_DCA_ORDERS};
    use astroport::asset::{Asset, AssetInfo, ULUNA_DENOM};
    use astroport::pair::DEFAULT_SLIPPAGE;
    use astroport_dca::dca::{Balance, DcaInfo};
    use cosmwasm_std::{
        coin,
        testing::{mock_info, MockStorage},
        Addr, Decimal, MemoryStorage, Uint128,
    };
    use std::str::FromStr;

    pub fn mock_storage_valid_data() -> MemoryStorage {
        let config = mock_config();

        // save CONFIG to storage
        let mut store = MockStorage::new();
        _ = CONFIG.save(&mut store, &config);

        // save USER_DCA_ORDERS to storage
        // save USER_DCA_ORDERS to storage
        let dca_orders = mock_dca_orders_valid_data();
        let funds = [coin(15, "usdt"), coin(100, "uluna")];
        let info = mock_info("creator", &funds);

        _ = USER_DCA_ORDERS.save(
            &mut store,
            &info.sender,
            &vec![dca_orders[0].id(), dca_orders[1].id()],
        );
        _ = DCA_ORDERS.save(&mut store, dca_orders[0].id(), &dca_orders[0]);
        _ = DCA_ORDERS.save(&mut store, dca_orders[1].id(), &dca_orders[1]);

        return store;
    }

    pub fn mock_config() -> Config {
        return Config {
            owner: Addr::unchecked("owner_addr"),
            max_hops: 3u32,
            max_spread: Decimal::from_str(DEFAULT_SLIPPAGE).unwrap(),
            per_hop_fee: Uint128::from(100u128),
            gas_info: AssetInfo::NativeToken {
                denom: ULUNA_DENOM.to_string(),
            },
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
                        contract_addr: Addr::unchecked("axlusdc"),
                    },
                    AssetInfo::Token {
                        contract_addr: Addr::unchecked("tip_2_addr"),
                    },
                ],
            },
            factory_addr: Addr::unchecked("factory_addr"),
            router_addr: Addr::unchecked("router_addr"),
        };
    }

    pub fn mock_dca_orders_valid_data() -> Vec<DcaInfo> {
        return vec![dca_order_1_valid(), dca_order_2_valid()];
    }

    pub fn dca_order_1_valid() -> DcaInfo {
        let dac_amount = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(10u128),
        };

        let balance = Balance {
            source: Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            spent: Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(0u128),
            },
            target: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("target_1_addr"),
                },
                amount: Uint128::from(10u128),
            },
            tip: Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(5u128),
            },
            gas: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: Uint128::from(5u128),
            },
            last_purchase: 0u64,
        };

        return DcaInfo::new(
            "order_1".to_string(),
            Addr::unchecked("creator"),
            10u64,
            50u64,
            10u64,
            dac_amount,
            None,
            None,
            balance,
        );
    }

    pub fn dca_order_2_valid() -> DcaInfo {
        let dac_amount = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("ibc/usdc"),
            },
            amount: Uint128::from(10u128),
        };

        let balance = Balance {
            source: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("ibc/usdc"),
                },
                amount: Uint128::from(800u128),
            },
            spent: Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(200u128),
            },
            target: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("target_2_addr"),
                },
                amount: Uint128::from(10u128),
            },
            tip: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("tip_2_addr"),
                },
                amount: Uint128::from(1000u128),
            },
            gas: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: Uint128::from(5u128),
            },
            last_purchase: 0u64,
        };

        return DcaInfo::new(
            "order_2".to_string(),
            Addr::unchecked("creator"),
            10u64,
            100u64,
            10u64,
            dac_amount,
            None,
            None,
            balance,
        );
    }
}
