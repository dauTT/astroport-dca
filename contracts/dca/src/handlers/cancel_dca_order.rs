use crate::{
    error::ContractError,
    state::{DCA_ORDERS, USER_DCA_ORDERS},
};
use astroport::asset::AssetInfo;
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

/// ## Description
/// Cancels a users DCA purchase so thatto_string() it will no longer be fulfilled.
///
/// Returns the `initial_asset` back to the user if it was a native token.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to cancel their order.
///
/// * `initial_asset` The [`AssetInfo`] which the user wants to cancel the DCA order for.
pub fn cancel_dca_order(
    deps: DepsMut,
    info: MessageInfo,
    id: String, // initial_asset: AssetInfo,
) -> Result<Response, ContractError> {
    let order = DCA_ORDERS.load(deps.as_ref().storage, id.clone())?;
    // permission check
    if info.sender != order.created_by() {
        return Err(ContractError::Unauthorized {});
    }

    remove_dca_order(deps, info.clone(), id.clone())?;
    let messages = build_refund_messages(info.clone(), order)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![attr("action", "cancel_dca_order"), attr("id", id)]))
}

// remove the order from the storage
fn remove_dca_order(deps: DepsMut, info: MessageInfo, id: String) -> Result<(), ContractError> {
    USER_DCA_ORDERS.update(
        deps.storage,
        &info.sender,
        |dca_orders: Option<Vec<String>>| -> Result<Vec<String>, ContractError> {
            let orders = &mut dca_orders.unwrap();

            for (index, o) in orders.iter().enumerate() {
                if o == &id {
                    orders.remove(index);
                    break;
                }
            }

            Ok(orders.clone())
        },
    )?;

    DCA_ORDERS.remove(deps.storage, id);

    Ok(())
}

fn build_refund_messages(
    info: MessageInfo,
    order: DcaInfo,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut refund_messages: Vec<CosmosMsg> = Vec::new();

    for asset in vec![
        order.balance.source,
        order.balance.tip,
        order.balance.gas,
        order.balance.target,
    ] {
        if asset.amount.gt(&Uint128::zero()) {
            match asset.info {
                AssetInfo::Token { contract_addr } => refund_messages.push(
                    WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        funds: vec![],
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: info.sender.to_string(),
                            amount: asset.amount,
                        })?,
                    }
                    .into(),
                ),
                AssetInfo::NativeToken { denom } => refund_messages.push(
                    BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: vec![Coin {
                            amount: asset.amount,
                            denom: denom.to_string(),
                        }],
                    }
                    .into(),
                ),
            }
        }
    }

    Ok(refund_messages)
}

#[cfg(test)]
mod tests {
    use crate::state::{DCA_ORDERS, USER_DCA_ORDERS};
    use astroport_dca::dca::ExecuteMsg;
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Empty, Response,
    };

    use super::super::deposit::test_util::mock_storage_valid_data;
    use crate::contract::execute;

    #[test]
    // deposit assets are whitelisted
    fn test_add_bot_tip_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "ibc/usdx")];
        let info = mock_info("creator", &funds);

        let dca_info_id = "order_2";
        // build msg
        // increment the uluna tip asset of 100 uluna of dca order 2
        let msg = ExecuteMsg::CancelDcaOrder {
            id: dca_info_id.to_string(),
        };

        // before removing dca_info_id
        let order = DCA_ORDERS
            .may_load(deps.as_ref().storage, dca_info_id.to_string())
            .unwrap();
        assert_ne!(order, None);

        let user_dca_orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap();
        assert_eq!(user_dca_orders.len(), 2);

        // execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // after removing dca_info_id
        let order = DCA_ORDERS
            .may_load(deps.as_ref().storage, dca_info_id.to_string())
            .unwrap();
        assert_eq!(order, None);

        let user_dca_orders = USER_DCA_ORDERS
            .load(deps.as_ref().storage, &info.sender)
            .unwrap();
        assert_eq!(user_dca_orders.len(), 1);

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "cancel_dca_order"),
            attr("id", "order_2"),
        ]);

        assert_eq!(actual_response.attributes, expected_response.attributes);
    }
}
