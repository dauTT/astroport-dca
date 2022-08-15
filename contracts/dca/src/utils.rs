use crate::error::ContractError;
use astroport::{
    asset::{Asset, AssetInfo},
    querier::{query_balance, query_token_balance},
};
use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, DepsMut, MessageInfo, QuerierWrapper, WasmMsg,
};
use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};
use cw20::Cw20ExecuteMsg;
use cw20::{AllowanceResponse, Cw20QueryMsg};
use std::collections::HashMap;

/// ## Description
/// Retrieves the allowed token allowance for the contract for a Cw20 token as a [`Uint128`].
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `owner` - The user that holds the Cw20 token.
///
/// * `contract_address` - The address of the Cw20 token.
pub fn get_token_allowance(
    deps: &Deps,
    env: &Env,
    owner: &Addr,
    contract_address: &Addr,
) -> StdResult<Uint128> {
    let allowance_response: AllowanceResponse = deps.querier.query_wasm_smart(
        contract_address,
        &Cw20QueryMsg::Allowance {
            owner: owner.to_string(),
            spender: env.contract.address.to_string(),
        },
    )?;

    Ok(allowance_response.allowance)
}

/// ## Description
/// Retrieves the native/token balance wrapped in a [`Asset`] structure for an account.
/// ## Arguments
/// * `querier` - A [`&QuerierWrapper`] that allow to execute queries.
///
/// * `account_addr` - The [`Addr`] which is the address of an account (user/contract).
///
/// * `asset_info` - The [`AssetInfo`] that contains the information of the asset.
pub fn query_asset_balance(
    querier: &QuerierWrapper,
    account_addr: Addr,
    asset_info: AssetInfo,
) -> StdResult<Asset> {
    let amount = match asset_info.clone() {
        AssetInfo::NativeToken { denom } => query_balance(querier, account_addr, denom),
        AssetInfo::Token { contract_addr } => {
            query_token_balance(querier, contract_addr, account_addr)
        }
    }?;

    Ok(Asset {
        info: asset_info,
        amount: amount,
    })
}

// try to subtract asset2 from asset1
pub fn try_sub(asset1: Asset, asset2: Asset) -> Result<Asset, ContractError> {
    if asset1.info != asset2.info {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "Expetected  asset1.info:{:?} and asset2.info{:?} to be consistent",
                asset1.info, asset2.info
            ),
        });
    };

    if asset2.amount.gt(&asset1.amount) {
        return Err(ContractError::InvalidInput {
            msg: format!(
                "Asset2.amount ({:?}) has to be smallet than asset1.amount ({:?}). Got asset2.amount > asset1.amount ",
                asset2.amount, asset1.amount
            ),
        });
    }

    let diff_asset_amount = asset1.amount.checked_sub(asset2.amount)?;
    let diff_asset = Asset {
        info: asset1.info,
        amount: diff_asset_amount,
    };

    Ok(diff_asset)
}

pub fn aggregate_assets(asset_map: &mut HashMap<String, Asset>, asset: Asset) {
    let key = asset.info.to_string();
    let op = asset_map.get(&key);
    match op {
        None => {
            asset_map.insert(key.clone(), asset.clone());
        }
        Some(a) => {
            let aggregated_amount = asset.amount.checked_add(a.amount).unwrap();
            let aggregated_asset = Asset {
                info: asset.info.clone(),
                amount: aggregated_amount,
            };

            asset_map.insert(key.clone(), aggregated_asset);
        }
    }
}

pub fn validate_all_deposit_assets(
    deps: &DepsMut,
    env: &Env,
    info: &MessageInfo,
    asset_map: &mut HashMap<String, Asset>,
) -> Result<(), ContractError> {
    for (_, asset) in asset_map {
        // check that user has sent the valid tokens to the contract
        // if native token, they should have included it in the message
        // otherwise, if cw20 token, they should have provided the correct allowance
        match &asset.info {
            AssetInfo::NativeToken { .. } => asset.assert_sent_native_token_balance(info)?,
            AssetInfo::Token { contract_addr } => {
                let allowance =
                    get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
                if allowance < asset.amount {
                    return Err(ContractError::AllowanceCheckFail {
                        token_addr: contract_addr.to_string(),
                        aggr_amount: asset.amount.to_string(),
                        allowance: allowance.to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

pub fn build_send_message(info: MessageInfo, asset: Asset) -> Result<CosmosMsg, ContractError> {
    let message: CosmosMsg = match asset.info.clone() {
        AssetInfo::Token { contract_addr } => WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: asset.amount,
            })?,
        }
        .into(),
        AssetInfo::NativeToken { denom } => BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                amount: asset.amount,
                denom: denom.to_string(),
            }],
        }
        .into(),
    };

    Ok(message)
}
