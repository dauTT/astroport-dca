use astroport::asset::{Asset, AssetInfo};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport_dca::dca::{DcaInfo, WhitelistedTokens};

/// Stores the main dca module parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract address that used for controls settings
    pub owner: Addr,
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_hops: u32,
    /// The maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_spread: Decimal,
    /// The fee a user must pay per hop performed in a DCA purchase when DCAing if the user does not specify.
    /// We assume the fee pay to the bot is a stablecoin denominated token in the (tip) Whitelist
    pub per_hop_fee: Uint128,
    // the denomination of the native gas asset of chain.
    // In terra is uluna, in Juno is ujuno and so on..
    pub gas_info: AssetInfo,
    // The list of tokens which are allowed in the DCA contracts.
    pub whitelisted_tokens: WhitelistedTokens,
    /// The address of the Astroport factory contract
    pub factory_addr: Addr,
    /// The address of the Astroport router contract
    pub router_addr: Addr,
}

/// The contract configuration
pub const CONFIG: Item<Config> = Item::new("config");
/// The DCA orders for a user.
/// The key is the user address and the value is the corresponding list of DCA order id.
pub const USER_DCA_ORDERS: Map<&Addr, Vec<String>> = Map::new("USER_DCA_ORDERS");
// The DCA orders. The key is the DCA order id and the value is the information of DCA.
// (For technical reason we can use Uint128 as key but instead use a String).
pub const DCA_ORDERS: Map<String, DcaInfo> = Map::new("DCA_ORDERS");
// Every time a user create a DCA order, this variable will increase of 1.
// This is needed create a unique id for the DCA orders.
pub const LAST_DCA_ORDER_ID: Item<String> = Item::new("LAST_DCA_ORDER_ID");
// Variable to store the temporare target balance of the DCA contract for a specific dca_order_id
// first element of the tuple: dca_order_id
// second element of the tuple : contract target balance
pub const TMP_CONTRACT_TARGET_BALANCE: Item<Option<(String, Asset)>> =
    Item::new("tmp_contract_target_balance");
