use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    WasmMsg,
};

use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::state::{CONFIG, USER_DCA_ORDERS};

/// ## Description
/// Adds a tip to the contract for a users DCA purchases.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] which contains a uusd tip to add to a users tip balance.
pub fn add_bot_tip(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dca_info_id: String,
    asset: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_tokens = config.whitelist_tokens;

    // Check tip asset is in the whitelist
    if !whitelist_tokens.is_tip_asset(&asset.info) {
        return Err(ContractError::InvalidInput {
            msg: format!("Tip asset, {:?},  not whitelisted", &asset.info),
        });
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    let asset_clone = asset.clone();
    match asset_clone.info {
        AssetInfo::NativeToken { denom: _ } => {
            asset.assert_sent_native_token_balance(&info)?;
        }
        AssetInfo::Token { contract_addr } => {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: asset_clone.amount,
                })?,
                funds: vec![],
            }));
        }
    }

    // debug_assert_eq!(format!("{:?}", asset.clone()), "1 XXXXXXXXXXXXXXXXXXXXXXX");

    USER_DCA_ORDERS.update(
        deps.storage,
        &info.sender,
        |config| -> StdResult<Vec<DcaInfo>> {
            let mut config = config.unwrap_or_default();
            for dca_info in &mut config {
                if dca_info.id() == dca_info_id {
                    for a in &mut dca_info.tip_assets {
                        if a.info == asset.info {
                            a.amount = a.amount.checked_add(asset.amount).unwrap();
                            return Ok(config);
                        }
                    }
                }
            }

            Ok(config)
        },
    )?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "add_bot_tip"),
        attr("dca_info_id", dca_info_id),
        attr("asset", format!("{:?}", asset)),
    ]))
}

#[cfg(test)]
mod tests {
    use crate::state::USER_DCA_ORDERS;
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::ExecuteMsg;
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Response, Uint128,
    };

    use super::test_util::mock_storage; //::mock_storage;
    use crate::contract::execute;

    #[test]
    // deposit assets are whitelisted
    fn test_add_bot_tip_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage();

        let funds = [coin(100, "uluna")];
        let info = mock_info("creator", &funds);

        let tip_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
            amount: Uint128::from(100u128),
        };
        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::AddBotTip {
            dca_info_id: "2".to_string(),
            asset: tip_asset.clone(),
        };

        // execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let dac_orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap_or_default();

        assert_eq!(2, dac_orders.len());

        let order_2 = dac_orders.iter().find(|d| d.id() == "2").unwrap();

        let expected_tip_asset = vec![
            // amount incremented of 100 after executing AddBotTip msg
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: Uint128::from(110u128),
            },
            // same as before executin the AddBotTip msg
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(5u128),
            },
        ];

        assert_eq!(order_2.tip_assets, expected_tip_asset);

        // let dca_0 {iy}= orders[0].clone();
        let expected_response = Response::new().add_attributes(vec![
            attr("action", "add_bot_tip"),
            attr("dca_info_id", "2"),
            attr("asset", format!("{:?}", tip_asset)),
        ]);

        assert_eq!(actual_response, expected_response);
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::state::{Config, WhitelistTokens, CONFIG, USER_DCA_ORDERS};
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::{DcaInfo, PurchaseSchedule};
    use cosmwasm_std::{
        coin,
        testing::{mock_info, MockStorage},
        Addr, MemoryStorage, Uint128,
    };

    pub fn mock_storage() -> MemoryStorage {
        let config = mock_config();

        // save CONFIG to storage
        let mut store = MockStorage::new();
        _ = CONFIG.save(&mut store, &config);

        // save USER_DCA_ORDERS to storage
        let user_dca_orders = mock_user_dca_orders();
        let funds = [coin(15, "usdt"), coin(100, "uluna")];
        let info = mock_info("creator", &funds);
        _ = USER_DCA_ORDERS.save(&mut store, &info.sender, &user_dca_orders);

        return store;
    }

    pub fn mock_config() -> Config {
        return Config {
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
                    AssetInfo::NativeToken {
                        denom: "uluna".to_string(),
                    },
                ],
            },
            factory_addr: Addr::unchecked("XXX"),
            router_addr: Addr::unchecked("YYY"),
        };
    }

    pub fn mock_user_dca_orders() -> Vec<DcaInfo> {
        // define DCA order 1
        let deposit_assets_1 = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(10u128),
        }];

        let tip_assets_1 = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(5u128),
        }];

        let target_asset_1 = AssetInfo::Token {
            contract_addr: Addr::unchecked("XXX"),
        };

        let gas_1 = Asset {
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
            amount: Uint128::from(100u128),
        };

        let purchase_schedules_1 = vec![PurchaseSchedule {
            asset_info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(10u128),
            interval: 100u64,
        }];

        let id_1 = "1".to_string();
        let dca1 = DcaInfo::new(
            id_1,
            10u64,
            10u64,
            10u64,
            target_asset_1,
            0u64,
            deposit_assets_1,
            tip_assets_1,
            gas_1,
            purchase_schedules_1,
            None,
            None,
        );

        // define DCA order 2
        let deposit_assets_2 = vec![
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(20u128),
            },
            Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0"),
                },
                amount: Uint128::from(100u128),
            },
        ];

        let tip_assets_2 = vec![
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: Uint128::from(10u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(5u128),
            },
        ];

        let target_asset_2 = AssetInfo::Token {
            contract_addr: Addr::unchecked("XXX"),
        };

        let gas_2 = Asset {
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
            amount: Uint128::from(200u128),
        };

        let purchase_schedules_2 = vec![PurchaseSchedule {
            asset_info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(20u128),
            interval: 200u64,
        }];

        let id_2 = "2".to_string();
        let dca2 = DcaInfo::new(
            id_2,
            10u64,
            10u64,
            10u64,
            target_asset_2,
            0u64,
            deposit_assets_2,
            tip_assets_2,
            gas_2,
            purchase_schedules_2,
            None,
            None,
        );

        return vec![dca1, dca2];
    }
}
