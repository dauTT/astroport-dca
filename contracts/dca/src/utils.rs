use crate::state::{Config, CONFIG};
use crate::{
    error::ContractError,
    get_token_allowance::get_token_allowance,
    state::{DCA_ORDERS, USER_DCA_ORDERS},
};

use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::{Balance, DcaInfo, WhitelistedTokens};
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, MessageInfo, Response, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
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

