use crate::error::ContractError;
use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, MessageInfo, WasmMsg};
use cw20::Cw20ExecuteMsg;

use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};
use cw20::{AllowanceResponse, Cw20QueryMsg};

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

/*

// check deposit asset is in the Whitelist
if !whitelisted_tokens.is_source_asset(&deposit.info) {
    return Err(ContractError::InvalidInput {
        msg: format!("Deposited asset, {:?},  not whitelisted", deposit.info),
    });
}

// check tip asset is whitelisted
if !whitelisted_tokens.is_tip_asset(&tip.info) {
    return Err(ContractError::InvalidInput {
        msg: format!(" tip asset, {:?},  not whitelisted", tip.info),
    });
}

*/

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
