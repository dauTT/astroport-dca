use crate::error::ContractError;
use crate::state::CONFIG;
use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

/// ## Description
/// Deposit assets into the DCA contract
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `asset` - Asset [`DepsMut`] that contains the dependencies.
pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<Asset>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_tokens = config.whitelist_tokens;

    let s = format!("{:?}", whitelist_tokens);
    for asset in assets.iter() {
        // check asset is in the WhiteList
        if !whitelist_tokens.is_deposit_asset(&asset.info) {
            return Err(StdError::generic_err(format!(
                "Asset {:?} not in the whitelist",
                asset.info
            ))
            .into());
        }
        // Native tokens live inside the underlying cosmos sdk bank module and
        // cw20 token lives inside its contract.
        // For native token we check if they have been sent.
        // (For token contract, the DCA contract will execute TransferFrom)
        asset.assert_sent_native_token_balance(&info)?;
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, asset) in assets.iter().enumerate() {
        // If the asset is token contract, then we need to execute TransferFrom msg to receive funds
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
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "deposit"),
        attr("assets", format!("{:?}", assets)),
    ]))
}

#[cfg(test)]
mod tests {
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::ExecuteMsg;

    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response, Uint128,
    };

    use super::super::add_bot_tip::test_util::mock_storage_valid_data;
    use crate::contract::execute;

    #[test]
    // deposit assets are whitelisted
    fn test_deposit_pass() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);

        // build msg
        let assets = vec![
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdt".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0"),
                },
                amount: Uint128::from(100u128),
            },
        ];
        let msg = ExecuteMsg::Deposit {
            assets: assets.clone(),
        };

        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let expected_response = Response::new().add_attributes(vec![
            attr("action", "deposit"),
            attr("assets", format!("{:?}", assets)),
        ]);
        assert_eq!(actual_response, expected_response)
    }

    #[test]
    fn test_deposit_token_unsupported() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);

        // build msg
        let assets = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: "xxx".to_string(),
            },
            amount: Uint128::from(100u128),
        }];

        let msg = ExecuteMsg::Deposit {
            assets: assets.clone(),
        };

        // execute the deposit msg
        let actual_response = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(actual_response.is_err(), true);
        let err = actual_response.err().unwrap();
        assert_eq!(
            err.to_string(),
            "Generic error: Asset NativeToken { denom: \"xxx\" } not in the whitelist"
        );
    }

    #[test]
    fn test_deposit_native_token_amount_mismatch() {
        // setup test
        let mut deps = mock_dependencies();
        deps.storage = mock_storage_valid_data();

        let funds = [coin(100, "usdt")];
        let info = mock_info("creator", &funds);

        // build msg
        let assets = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: "usdt".to_string(),
            },
            amount: Uint128::from(1u128),
        }];
        let msg = ExecuteMsg::Deposit {
            assets: assets.clone(),
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
}
