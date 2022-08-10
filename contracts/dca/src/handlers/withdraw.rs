use crate::{error::ContractError, state::DCA_ORDERS, utils::build_send_message};
use astroport::asset::Asset;
use astroport_dca::dca::{find_asset_info, DcaAssetType, DcaInfo};
use cosmwasm_std::{attr, DepsMut, MessageInfo, Response};

/// ## Description
/// Withdraws a users bot tip from the contract.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to withdraw their bot tip.
///
/// * `amount`` - A [`Uint128`] representing the amount of uusd to send back to the user.
pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    withdraw_type: DcaAssetType,
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
            let new_order = try_sub(dca_order, withdraw_type.clone(), asset.clone())?;
            Ok(new_order.clone())
        },
    )?;

    let message = build_send_message(info.clone(), asset.clone())?;

    Ok(Response::new().add_message(message).add_attributes(vec![
        attr("action", "withdraw"),
        attr("withdraw_type", format!("{:?}", withdraw_type)),
        attr("dca_order_id", dca_order_id),
        attr("asset", format!("{:?}", asset)),
    ]))
}

pub fn try_sub(
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
            order.balance.source.amount = order.balance.source.amount.checked_sub(asset.amount)?
        }
        DcaAssetType::Spent => {
            return Err(ContractError::InvalidInput {
                msg: format!("asset_type '{:?}' is not allowed", DcaAssetType::Spent),
            })
        }
        DcaAssetType::Target => {
            order.balance.target.amount = order.balance.target.amount.checked_sub(asset.amount)?
        }
        DcaAssetType::Tip => {
            order.balance.tip.amount = order.balance.tip.amount.checked_sub(asset.amount)?
        }
        DcaAssetType::Gas => {
            order.balance.gas.amount = order.balance.gas.amount.checked_sub(asset.amount)?
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

        let funds = [coin(10, "usdt")];
        let info = mock_info("creator", &funds);

        // build msg
        let withdraw_type = DcaAssetType::Source;
        let dac_order_id = "1".to_string();
        let asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(10u128),
        };

        let msg = ExecuteMsg::Withdraw {
            withdraw_type: withdraw_type.clone(),
            dca_order_id: "1".to_string(),
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

        assert_eq!(order.balance.source.amount, Uint128::from(90u128));

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "withdraw"),
            attr("withdraw_type", format!("{:?}", DcaAssetType::Source)),
            attr("dca_order_id", dac_order_id),
            attr("asset", format!("{:?}", asset)),
        ]);
        assert_eq!(actual_response.attributes, expected_response.attributes);
    }

    #[test]
    fn test_withdraw_target_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "1";

        // build msg
        let withdraw_type = DcaAssetType::Target;
        let asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("target_1_addr"),
            },
            amount: Uint128::from(2u128),
        };

        let msg = ExecuteMsg::Withdraw {
            withdraw_type: withdraw_type.clone(),
            dca_order_id: dac_order_id.to_string(),
            asset: asset.clone(),
        };

        //Check  amount before execution
        let mut order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.to_string())
            .unwrap();

        assert_eq!(order.balance.target.amount, Uint128::from(10u128));

        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "withdraw"),
            attr("withdraw_type", format!("{:?}", DcaAssetType::Target)),
            attr("dca_order_id", dac_order_id),
            attr("asset", format!("{:?}", asset)),
        ]);

        //Check  amount before execution
        order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.to_string())
            .unwrap();

        assert_eq!(order.balance.target.amount, Uint128::from(8u128));
        assert_eq!(actual_response.attributes, expected_response.attributes);
    }

    #[test]
    fn test_withdraw_invalid_tip_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "1".to_string();

        // build msg
        let withdraw_type = DcaAssetType::Source;
        let asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("tip_1_addr"),
            },
            amount: Uint128::from(1u128),
        };
        let msg = ExecuteMsg::Withdraw {
            withdraw_type: withdraw_type.clone(),
            dca_order_id: dac_order_id,
            asset: asset.clone(),
        };
        // execute the withdraw msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(actual_response.is_err(), true);
        let err = actual_response.err().unwrap();
        assert_eq!(
            err.to_string(),
            "Invalid input. msg: 'Expetected Source asset info:NativeToken { denom: \"usdt\" }.  Got: Token { contract_addr: Addr(\"tip_1_addr\") }'"
        );
    }

    #[test]
    fn test_deposit_tip_asset_invalid_amount() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "ibc/usdx"), coin(50, "usdt")];
        let info = mock_info("creator", &funds);
        let dac_order_id = "1".to_string();

        // build msg
        let withdraw_type = DcaAssetType::Tip;
        let tip_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(1000u128),
        };

        //Check  amount before execution
        let order = DCA_ORDERS
            .load(deps.as_ref().storage, dac_order_id.clone())
            .unwrap();
        assert_eq!(order.balance.tip.amount, Uint128::from(5u128));

        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::Withdraw {
            withdraw_type: withdraw_type.clone(),
            dca_order_id: dac_order_id.clone(),
            asset: tip_asset.clone(),
        };
        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(actual_response.is_err(), true);
        let err = actual_response.err().unwrap();
        assert_eq!(err.to_string(), "Cannot Sub with 5 and 1000");
    }
}
