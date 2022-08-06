use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::{DcaInfo, WhitelistedTokens};
use cosmwasm_std::{
    attr, coins, BankMsg, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
};

use crate::{
    error::ContractError,
    state::{CONFIG, DCA_ORDERS},
    utils::{build_send_message, get_token_allowance},
};

#[derive(Clone, Debug)]
/// Stores a modified dca order new parameters
pub struct ModifyDcaOrderParameters {
    /// The new [`Asset`] that is being spent to create DCA orders.
    pub new_source_asset: Option<Asset>,
    /// The [`AssetInfo`] that is being purchased with `new_source_asset`.
    pub new_target_asset_info: Option<AssetInfo>,
    /// The time in seconds between DCA purchases.
    pub new_tip_asset: Option<Asset>,
    /// The time in seconds between DCA purchases.
    pub new_interval: Option<u64>,
    /// a [`Uint128`] amount of `new_source_asset` to spend each DCA purchase.
    pub new_dca_amount: Option<Asset>,
    /// The new start time of the DCA. Prior to this time no purchases will be done.
    pub new_start_at: Option<u64>,
    /// The new maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing
    pub new_max_hops: Option<u32>,
    /// The new maximum amount of spread when performing a swap from `source_asset` to `target_asset` when DCAing
    pub new_max_spread: Option<Decimal>,
}

/// ## Description
/// Modifies an existing DCA order for a user such that the new parameters will apply to the
/// existing order.
///
/// If the user increases the size of their order, they must allocate the correct amount of new
/// assets to the contract.
///
/// If the user decreases the size of their order, they will be refunded with the difference.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to modify their order, containing the
/// [`AssetInfo::NativeToken`] if the DCA order is being increased in size.
///
/// * `order_details` - The [`ModifyDcaOrderParameters`] details about the old and new DCA order
/// parameters.
pub fn modify_dca_order(
    deps: DepsMut,
    info: MessageInfo,
    dca_order_id: String,
    change_request: ModifyDcaOrderParameters,
) -> Result<Response, ContractError> {
    let ModifyDcaOrderParameters {
        new_source_asset,
        new_target_asset_info,
        new_tip_asset,
        new_interval,
        new_dca_amount,
        new_start_at,
        new_max_hops,
        new_max_spread,
    } = change_request.clone();
    let order = &mut DCA_ORDERS.load(deps.as_ref().storage, dca_order_id.clone())?;
    let config = CONFIG.load(deps.storage)?;
    let whitelisted_tokens = config.whitelisted_tokens;

    let mut messages: Vec<CosmosMsg> = Vec::new();

    if let Some(new_source) = new_source_asset {
        // Check new_source_asset info is not the same as the current source asset info
        // Check new_source_asset is whitelisted
        //Check new_dca_amount is consistent with new_source_asset:
        //      i) new_dca_amount is not None
        //      ii) Check new_dca_amount.info = order.balance.source.info
        //      iii)  Check new_dca_amount > 0
        // Replace the current source_asset with new source_asset
        // Reset balance of spent_asset
        // Build a send msg to return the current source_asset to the user
        let msg = replace_source_asset(
            info.clone(),
            whitelisted_tokens.clone(),
            order,
            new_source.clone(),
            new_dca_amount.clone(),
        )?;
        messages.push(msg);
    }

    if let Some(new_target_asset_info) = new_target_asset_info {
        // Check new_target_asset info is not the same as the current target asset info
        // Replace the current target with new_target_asset
        // Reset balance of spent, target_asset
        // Build a send msg to return the current target_asset to the user
        let msg = replace_target_asset(info.clone(), order, new_target_asset_info)?;
        messages.push(msg);
    }

    if let Some(new_tip) = new_tip_asset {
        // Check new_tip_asset info is not the same as the current tip asset info
        // Check new_tip_asset is whitelisted
        // Replace the current tip_asset with new_tip_asset
        // Build a send msg to return the current source_asset to the user
        let msg = replace_tip_asset(info.clone(), whitelisted_tokens, order, new_tip.clone())?;
        messages.push(msg);
    }

    if let Some(new_interval) = new_interval {
        order.interval = new_interval;
    }

    if let Some(new_dca_amount) = new_dca_amount {
        // Check new_dca_amount info same as the source asset info
        // Replace the dca_amount with new_dca_amount
        replace_dca_amount(order, new_dca_amount)?
    }

    if let Some(new_start_at) = new_start_at {
        order.start_at = new_start_at;
    }

    if let Some(_) = new_max_hops {
        order.max_hops = new_max_hops;
    }

    if let Some(_) = new_max_spread {
        order.max_spread = new_max_spread;
    }

    DCA_ORDERS.save(deps.storage, dca_order_id.clone(), order)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "modify_dca_order"),
        attr("dca_order_id", dca_order_id.to_string()),
        attr("change_request", format!("{:?}", change_request)),
    ]))
}

fn replace_source_asset(
    info: MessageInfo,
    whitelisted_tokens: WhitelistedTokens,
    order: &mut DcaInfo,
    new_source_asset: Asset,
    new_dca_amount: Option<Asset>,
) -> Result<CosmosMsg, ContractError> {
    // Check new_source_asset info is not the same as the current source asset
    if order.balance.source.info == new_source_asset.info {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "New source asset info, {:?}, must be different from the current source asset info",
                &new_source_asset.info
            ),
        });
    };
    // Check it is whitelisted
    if !whitelisted_tokens.is_source_asset(&new_source_asset.info) {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "Source asset, {:?},  not whitelisted",
                &new_source_asset.info
            ),
        });
    };

    //Check new_dca_amount is consistent with new_source_asset:
    // i) new_dca_amount is not None
    // ii) Check new_dca_amount.info = order.balance.source.info
    // iii)  Check new_dca_amount > 0
    if new_dca_amount == None {
        return Err(ContractError::InvalidInput {
            msg: "Please provide a new_dca_amount consitent with the new_source_asset".to_string(),
        });
    }

    if let Some(new_dca_amount) = new_dca_amount {
        // Check new_dca_amount.info = order.balance.source.info
        if new_source_asset.info != new_dca_amount.info {
            return Err(ContractError::InvalidInput {
                msg: format!(
                    "New source asset info, ({:?}), not compatible with new_dca_amount info ({:?})",
                    new_source_asset.info, new_dca_amount.info
                ),
            });
        };

        // Check new_dca_amount > 0
        if new_dca_amount.amount.is_zero() {
            return Err(ContractError::InvalidInput {
                msg: format!("Expected new_dca_amount > 0. Got 0",),
            });
        };
    }

    // Replace the current source_asset with new source_asset
    order.balance.source = new_source_asset.clone();
    // Reset balance of spent_asset
    order.balance.spent = Asset {
        info: new_source_asset.info.clone(),
        amount: Uint128::zero(),
    };

    let msg = build_send_message(info, new_source_asset)?;
    Ok(msg)
}

fn replace_target_asset(
    info: MessageInfo,
    order: &mut DcaInfo,
    new_target_asset_info: AssetInfo,
) -> Result<CosmosMsg, ContractError> {
    let new_target_asset = Asset {
        info: new_target_asset_info.clone(),
        amount: Uint128::zero(),
    };
    // Check new_target_asset info is not the same as the current target asset info
    if order.balance.target.info == new_target_asset_info {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "new target asset info, {:?}, must be different from the current target asset info",
                new_target_asset_info
            ),
        });
    };
    // Replace the current target with new_target_asset
    order.balance.target = new_target_asset.clone();
    // Reset balance of spent, target_asset
    order.balance.spent.amount = Uint128::zero();
    // Reset last_purchase time
    order.balance.last_purchase = 0u64;
    // Build a send msg to return the current target_asset to the user
    let msg = build_send_message(info, new_target_asset)?;
    Ok(msg)
}

fn replace_tip_asset(
    info: MessageInfo,
    whitelisted_tokens: WhitelistedTokens,
    order: &mut DcaInfo,
    new_tip_asset: Asset,
) -> Result<CosmosMsg, ContractError> {
    // Check new_tip_asset info is not the same as the current tip asset info
    if order.balance.tip.info == new_tip_asset.info {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "new tip asset info, {:?}, must be different from the current tip asset info",
                &new_tip_asset.info
            ),
        });
    };
    // Check it is whitelisted
    if !whitelisted_tokens.is_tip_asset(&new_tip_asset.info) {
        return Err(ContractError::InvalidInput {
            msg: format!("tip asset, {:?},  not whitelisted", &new_tip_asset.info),
        });
    };

    // Check tip amount > 0
    if new_tip_asset.amount.is_zero() {
        return Err(ContractError::InvalidInput {
            msg: "Exptected tip amount > 0 , got 0".to_string(),
        });
    };
    // Replace the tip_asset with new_tip_asset
    order.balance.tip = new_tip_asset.clone();

    let msg = build_send_message(info, new_tip_asset)?;
    Ok(msg)
}

fn replace_dca_amount(order: &mut DcaInfo, new_dca_amount: Asset) -> Result<(), ContractError> {
    // Check new_dca_amount info same as the source asset info
    if order.balance.source.info.clone() != new_dca_amount.info {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "new_dca_amount info,({:?}), not compatible with source asset info ({:?})",
                new_dca_amount.info,
                order.balance.source.info.clone()
            ),
        });
    };

    // Replace the dca_amount with new_dca_amount
    order.dca_amount = new_dca_amount;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        error::ContractError,
        state::{CONFIG, DCA_ORDERS},
    };
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::ExecuteMsg;

    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, DepsMut, Empty, Env, MessageInfo, Response, Uint128,
    };

    use super::ModifyDcaOrderParameters;
    use crate::contract::execute;
    use crate::fixture::fixture::{dca_order_1_valid, mock_storage_valid_data};

    fn build_msg(dca_info_id: String, change_request: ModifyDcaOrderParameters) -> ExecuteMsg {
        return ExecuteMsg::ModifyDcaOrder {
            id: dca_info_id.clone(),
            new_source_asset: change_request.new_source_asset.clone(),
            new_target_asset_info: change_request.new_target_asset_info.clone(),
            new_tip_asset: change_request.new_tip_asset.clone(),
            new_interval: change_request.new_interval.clone(),
            new_dca_amount: change_request.new_dca_amount.clone(),
            new_start_at: change_request.new_start_at.clone(),
            new_max_hops: change_request.new_max_hops.clone(),
            new_max_spread: change_request.new_max_spread.clone(),
        };
    }

    #[test]
    fn test_modify_dca_order_valid_source_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("owner_addr", &funds);

        // build msg
        let dca_info_id = "order_1".to_string();
        let new_source_asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset0"),
            },
            amount: Uint128::from(100u128),
        };
        let new_dca_amount = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset0"),
            },
            amount: Uint128::from(50u128),
        };

        let change_request = ModifyDcaOrderParameters {
            new_source_asset: Some(new_source_asset.clone()),
            new_target_asset_info: None,
            new_tip_asset: None,
            new_interval: None,
            new_dca_amount: Some(new_dca_amount.clone()),
            new_start_at: None,
            new_max_hops: None,
            new_max_spread: None,
        };

        let msg = build_msg(dca_info_id.clone(), change_request.clone());

        // before
        let order = DCA_ORDERS.load(&deps.storage, dca_info_id.clone()).unwrap();
        assert_eq!(order.balance.source, dca_order_1_valid().balance.source);
        assert_eq!(order.dca_amount, dca_order_1_valid().dca_amount);

        // Execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "modify_dca_order"),
            attr("dca_order_id", dca_info_id.clone()),
            attr("change_request", format!("{:?}", change_request)),
        ]);

        // after
        let order = DCA_ORDERS.load(&deps.storage, dca_info_id).unwrap();
        assert_eq!(order.balance.source, new_source_asset);
        assert_eq!(order.dca_amount, new_dca_amount);
        assert_eq!(actual_response.attributes, expected_response.attributes);
        // assert_eq!(format!("{:?}", actual_response.messages), "[SubMsg { id: 0, msg: Wasm(Execute { contract_addr: \"asset0\", msg: Binary(7b227472616e73666572223a7b22726563697069656e74223a226f776e65725f61646472222c22616d6f756e74223a22313030227d7d), funds: [] }), gas_limit: None, reply_on: Never }]");
    }

    #[test]
    fn test_modify_dca_order_valid_target_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("owner_addr", &funds);

        // build msg
        let dca_info_id = "order_1".to_string();
        let new_target_asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("target"),
            },
            amount: Uint128::zero(),
        };

        let change_request = ModifyDcaOrderParameters {
            new_source_asset: None,
            new_target_asset_info: Some(new_target_asset.info.clone()),
            new_tip_asset: None,
            new_interval: None,
            new_dca_amount: None,
            new_start_at: None,
            new_max_hops: None,
            new_max_spread: None,
        };

        let msg = build_msg(dca_info_id.clone(), change_request.clone());

        // before
        let order = DCA_ORDERS.load(&deps.storage, dca_info_id.clone()).unwrap();
        assert_eq!(order.balance.target, dca_order_1_valid().balance.target);
        assert_eq!(
            order.balance.spent.amount,
            dca_order_1_valid().balance.spent.amount
        );
        assert_eq!(
            order.balance.last_purchase,
            dca_order_1_valid().balance.last_purchase
        );

        // Execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // after
        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "modify_dca_order"),
            attr("dca_order_id", dca_info_id.clone()),
            attr("change_request", format!("{:?}", change_request)),
        ]);

        let order = DCA_ORDERS.load(&deps.storage, dca_info_id).unwrap();

        assert_eq!(order.balance.target, new_target_asset);
        assert_eq!(order.balance.spent.amount, Uint128::zero());
        assert_eq!(order.balance.last_purchase, 0u64);
        assert_eq!(actual_response.attributes, expected_response.attributes);
    }

    #[test]
    fn test_modify_dca_order_valid_tip_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("owner_addr", &funds);

        // build msg
        let dca_info_id = "order_1".to_string();
        let new_tip_asset = Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("axlusdc"),
            },
            amount: Uint128::from(100u128),
        };

        let change_request = ModifyDcaOrderParameters {
            new_source_asset: None,
            new_target_asset_info: None,
            new_tip_asset: Some(new_tip_asset.clone()),
            new_interval: None,
            new_dca_amount: None,
            new_start_at: None,
            new_max_hops: None,
            new_max_spread: None,
        };

        let msg = build_msg(dca_info_id.clone(), change_request.clone());

        // before
        let order = DCA_ORDERS.load(&deps.storage, dca_info_id.clone()).unwrap();
        assert_eq!(order.balance.tip, dca_order_1_valid().balance.tip);

        // Execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let expected_response: Response<Empty> = Response::new().add_attributes(vec![
            attr("action", "modify_dca_order"),
            attr("dca_order_id", dca_info_id.clone()),
            attr("change_request", format!("{:?}", change_request)),
        ]);

        let order = DCA_ORDERS.load(&deps.storage, dca_info_id).unwrap();

        assert_eq!(order.balance.tip, new_tip_asset);
        assert_eq!(actual_response.attributes, expected_response.attributes);
    }

    #[test]
    fn test_modify_dca_order_invalid_source_asset() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("owner_addr", &funds);

        // build msg
        let dca_info_id = "order_1".to_string();
        let new_source_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "new_usdt".to_string(),
            },
            amount: Uint128::from(100u128),
        };

        let change_request = ModifyDcaOrderParameters {
            new_source_asset: Some(new_source_asset.clone()),
            new_target_asset_info: None,
            new_tip_asset: None,
            new_interval: None,
            new_dca_amount: None,
            new_start_at: None,
            new_max_hops: None,
            new_max_spread: None,
        };

        let msg = build_msg(dca_info_id.clone(), change_request.clone());

        // Execute the msg
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        assert_eq!(err.to_string(), "Invalid input. msg: 'Source asset, NativeToken { denom: \"new_usdt\" },  not whitelisted'");
    }

    #[test]
    fn test_modify_dca_order_invalid_source_dca_amount() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("owner_addr", &funds);

        // build msg
        let dca_info_id = "order_1".to_string();
        let new_source_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
            amount: Uint128::from(100u128),
        };

        let new_dca_amount = Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(50u128),
        };

        let change_request = ModifyDcaOrderParameters {
            new_source_asset: Some(new_source_asset.clone()),
            new_target_asset_info: None,
            new_tip_asset: None,
            new_interval: None,
            new_dca_amount: Some(new_dca_amount.clone()),
            new_start_at: None,
            new_max_hops: None,
            new_max_spread: None,
        };

        let msg = build_msg(dca_info_id.clone(), change_request.clone());

        // Execute the msg
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        assert_eq!(err.to_string(), "Invalid input. msg: 'New source asset info, (NativeToken { denom: \"uluna\" }), not compatible with new_dca_amount info (NativeToken { denom: \"usdt\" })'");
    }
}
