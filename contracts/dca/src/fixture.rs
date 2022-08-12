#[cfg(test)]
pub mod fixture {
    use crate::state::{
        Config, CONFIG, DCA_ORDERS, LAST_DCA_ORDER_ID, TMP_CONTRACT_TARGET_GAS_BALANCE,
        USER_DCA_ORDERS,
    };
    use astroport::asset::{Asset, AssetInfo, ULUNA_DENOM};
    use astroport::pair::DEFAULT_SLIPPAGE;
    use astroport_dca::dca::{Balance, DcaInfo, WhitelistedTokens};
    use cosmwasm_std::{
        coin,
        testing::{mock_info, MockStorage},
        Addr, Decimal, MemoryStorage, Uint128,
    };
    use std::str::FromStr;

    pub fn mock_storage_valid_data() -> MemoryStorage {
        let config = mock_config();

        // save state
        let mut store = MockStorage::new();
        _ = CONFIG.save(&mut store, &config);
        _ = TMP_CONTRACT_TARGET_GAS_BALANCE.save(&mut store, &None);
        _ = LAST_DCA_ORDER_ID.save(&mut store, &"2".to_string());

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
            whitelisted_tokens: WhitelistedTokens {
                source: vec![
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
            last_purchase: 400u64,
        };

        return DcaInfo::new(
            "1".to_string(),
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
            last_purchase: 100u64,
        };

        return DcaInfo::new(
            "2".to_string(),
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
