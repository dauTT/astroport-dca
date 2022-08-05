use astroport::{asset::AssetInfo, querier::query_factory_config};
use cosmwasm_std::{attr, Addr, Decimal, DepsMut, MessageInfo, Response, StdError, Uint128};

use crate::{error::ContractError, state::CONFIG};

/// ## Description
/// Updates the contract configuration with the specified optional parameters.
///
/// If any new configuration value is excluded, the current configuration value will remain
/// unchanged.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the factory contract owner who wants to modify the
/// configuration of the contract.
///
/// * `max_hops` - An optional value which represents the new maximum amount of hops per swap if the
/// user does not specify a value.
///
/// * `per_hop_fee` - An optional [`Uint128`] which represents the new uusd fee paid to bots per hop
/// executed in a DCA purchase.
///
/// * `whitelisted_tokens` - An optional [`Vec<AssetInfo>`] which represents the new whitelisted
/// tokens that can be used in a hop route for DCA purchases.
///
/// * `max_spread` - An optional [`Decimal`] which represents the new maximum spread for each DCA
/// purchase if the user does not specify a value.
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    max_hops: Option<u32>,
    per_hop_fee: Option<Uint128>,
    whitelisted_tokens_deposit: Option<Vec<AssetInfo>>,
    whitelisted_tokens_tip: Option<Vec<AssetInfo>>,
    max_spread: Option<Decimal>,
    router_addr: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    CONFIG.update::<_, StdError>(deps.storage, |mut config| {
        if let Some(new_max_hops) = max_hops {
            config.max_hops = new_max_hops;
        }

        if let Some(new_per_hop_fee) = per_hop_fee {
            config.per_hop_fee = new_per_hop_fee;
        }

        if let Some(new_whitelisted_tokens_deposit) = whitelisted_tokens_deposit {
            config.whitelist_tokens.deposit = new_whitelisted_tokens_deposit;
        }

        if let Some(new_whitelisted_tokens_tip) = whitelisted_tokens_tip {
            config.whitelist_tokens.tip = new_whitelisted_tokens_tip;
        }

        if let Some(new_max_spread) = max_spread {
            config.max_spread = new_max_spread;
        }

        if let Some(new_router_addr) = router_addr {
            config.router_addr = new_router_addr;
        }

        Ok(config)
    })?;

    Ok(Response::default().add_attributes(vec![attr("action", "update_config")]))
}

#[cfg(test)]
mod tests {
    use crate::state::CONFIG;
    use astroport::asset::AssetInfo;
    use astroport_dca::dca::ExecuteMsg;

    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, Response,
    };

    use super::super::add_bot_tip::test_util::mock_storage_valid_data;
    use crate::contract::execute;

    #[test]
    // deposit assets are whitelisted
    fn test_update_config() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(200, "usdt"), coin(100, "uluna")];
        let info = mock_info("owner_addr", &funds);

        // build msg
        let new_max_hops = Some(10u32);
        let new_whitelisted_tokens_deposit = Some(vec![AssetInfo::NativeToken {
            denom: "usdt".to_string(),
        }]);

        let msg = ExecuteMsg::UpdateConfig {
            max_hops: new_max_hops.clone(),
            per_hop_fee: None,
            whitelisted_tokens_deposit: new_whitelisted_tokens_deposit.clone(),
            whitelisted_tokens_tip: None,
            max_spread: None,
            router_addr: None,
        };

        // execute the msg
        let actual_response = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let expected_response: Response<Empty> =
            Response::new().add_attributes(vec![attr("action", "update_config")]);

        let config = CONFIG.load(&deps.storage).unwrap();

        assert_eq!(actual_response, expected_response);
        assert_eq!(config.max_hops, new_max_hops.unwrap());
        assert_eq!(
            config.whitelist_tokens.deposit,
            new_whitelisted_tokens_deposit.unwrap()
        );
    }
}
