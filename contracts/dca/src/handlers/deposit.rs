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

    // check asset.amount > 0
    // (amount can never be negative)
    if asset.amount.is_zero() {
        return Err(ContractError::InvalidInput {
            msg: format!("Expetected  asset amount > 0. Got: 0",),
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
    use crate::fixture::fixture::mock_storage_valid_data;
    use crate::{contract::execute, state::DCA_ORDERS};
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::{DcaAssetType, ExecuteMsg};
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
