use astroport::asset::{Asset, AssetInfo, ULUNA_DENOM};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    WasmMsg,
};

use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::state::{CONFIG, DCA_ORDERS};

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

    DCA_ORDERS.update(
        deps.storage,
        dca_info_id.clone(),
        |order| -> Result<DcaInfo, ContractError> {
            let order = &mut order.ok_or(ContractError::NonexistentDca {
                msg: format! {"invalid dca order id: {}", dca_info_id},
            })?;

            order.balance.tip.amount = order.balance.tip.amount.checked_add(asset.amount).unwrap();
            return Ok(order.clone());
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
    use crate::state::{DCA_ORDERS, USER_DCA_ORDERS};
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::ExecuteMsg;
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, Response, Uint128,
    };

    use super::test_util::mock_storage_valid_data; //::mock_storage;
    use crate::contract::execute;

    #[test]
    // deposit assets are whitelisted
    fn test_add_bot_tip_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "ibc/usdx")];
        let info = mock_info("creator", &funds);

        let tip_asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("tip_2_addr"),
            },
            amount: Uint128::from(50u128),
        };

        let user_orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap();

        assert_eq!(user_orders.len(), 2);
        let dca_info_id = "order_2";

        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::AddBotTip {
            dca_info_id: dca_info_id.to_string(),
            asset: tip_asset.clone(),
        };

        // execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let order = DCA_ORDERS
            .load(deps.as_ref().storage, dca_info_id.to_string())
            .unwrap();

        let expected_balance_tip = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("tip_2_addr"),
            },
            amount: Uint128::from(150u128),
        };
        assert_eq!(order.balance.tip, expected_balance_tip);

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "add_bot_tip"),
            attr("dca_info_id", "order_2"),
            attr("asset", format!("{:?}", tip_asset)),
        ]);

        assert_eq!(actual_response.attributes, expected_response.attributes);
        assert_eq!(format!("{:?}", actual_response.messages), "[SubMsg { id: 0, msg: Wasm(Execute { contract_addr: \"tip_2_addr\", msg: Binary(7b227472616e736665725f66726f6d223a7b226f776e6572223a2263726561746f72222c22726563697069656e74223a22636f736d6f7332636f6e7472616374222c22616d6f756e74223a223530227d7d), funds: [] }), gas_limit: None, reply_on: Never }]")
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
        CONFIG.save(&mut store, &config);

        // save USER_DCA_ORDERS to storage
        // save USER_DCA_ORDERS to storage
        let dca_orders = mock_dca_orders_valid_data();
        let funds = [coin(15, "usdt"), coin(100, "uluna")];
        let info = mock_info("creator", &funds);

        USER_DCA_ORDERS.save(
            &mut store,
            &info.sender,
            &vec![dca_orders[0].id(), dca_orders[1].id()],
        );
        DCA_ORDERS.save(&mut store, dca_orders[0].id(), &dca_orders[0]);
        DCA_ORDERS.save(&mut store, dca_orders[1].id(), &dca_orders[1]);

        return store;
    }

    pub fn mock_config() -> Config {
        return Config {
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
            deposit: Asset {
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
            Addr::unchecked("user 1"),
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
            deposit: Asset {
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
                amount: Uint128::from(100u128),
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
            Addr::unchecked("user 2"),
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
