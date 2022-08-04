use std::ops::Add;

use astroport::{
    asset::{Asset, AssetInfo},
    router::SwapOperation,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};

/// Describes information about a DCA order
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DcaInfo {
    // Unique identifier
    id: String,
    // The address of the user who has created the dca order.
    created_by: Addr,
    // the time of the creation of of the dca order.
    created_at: u64,
    // the time when the dca will start to become active
    pub start_at: u64,
    /// The interval in seconds between DCA purchases
    pub interval: u64,
    /// The amount of (deposit) asset to spend at each DCA purchase of the target asset
    pub dca_amount: Asset,
    /// An override for the maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing
    pub max_hops: Option<u32>,
    /// An override for the maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing
    pub max_spread: Option<Decimal>,
    /// The balance of the assets involved in the DCA
    pub balance: Balance,
}
impl DcaInfo {
    pub const fn new(
        id: String,
        created_by: Addr,
        created_at: u64,
        start_at: u64,
        interval: u64,
        dca_amount: Asset,
        max_hops: Option<u32>,
        max_spread: Option<Decimal>,
        balance: Balance,
    ) -> Self {
        return DcaInfo {
            id,
            created_by,
            created_at,
            start_at,
            interval,
            dca_amount,
            max_hops,
            max_spread,
            balance,
        };
    }

    pub fn id(&self) -> String {
        self.id.to_owned()
    }

    pub fn created_by(&self) -> Addr {
        self.created_by.to_owned()
    }

    pub fn created_at(&self) -> u64 {
        self.created_at.to_owned()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Balance {
    /// The available asset in the DCA for purchasing the target asset.
    /// The balance of this asset will decrease of a dca_amount at each purchase.
    pub deposit: Asset,
    /// The deposited asset which have been already spent to purchase the target asset.
    /// The balance of this asset will increase of a dca_amount at each purchase.
    /// Note that at each purchase: deposit + spent  = constant
    pub spent: Asset,
    /// The asset which the user wants to buy with the DCA order.
    /// The balance of this asset will increase at each purchase.
    pub target: Asset,
    /// The tip amount the user has deposited for their tips when performing DCA purchases.
    /// The balance of this asset will decrease at each purchase.
    pub tip: Asset,
    /// The amount of gas token (uluna) the user has deposited for their swaps when performing DCA purchases.
    /// The balance of this asset will decrease at each purchase.
    pub gas: Asset,
    /// The last time the `target_asset` was purchased.
    /// This field will be updated at each purchase.
    pub last_purchase: u64,
}

/// Describes the parameters used for creating a contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if
    /// the user does not specify a custom max hop amount
    pub max_hops: u32,
    /// The fee a user must pay per hop performed in a DCA purchase
    pub per_hop_fee: Uint128,
    /// The whitelisted tokens that can be used in a DCA hop route
    pub whitelisted_tokens: Vec<AssetInfo>,
    /// The maximum amount of spread
    pub max_spread: String,
    /// The address of the Astroport factory contract
    pub factory_addr: String,
    /// The address of the Astroport router contract
    pub router_addr: String,
}

/// This structure describes the execute messages available in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Add uusd top-up for bots to perform DCA requests
    AddBotTip {
        // The unique Id in DCA order
        dca_info_id: String,
        // An asset in the whitelist of type 'tip', which willl be use ti reward the bot.
        asset: Asset,
    },
    /// Cancels a DCA order, returning any native asset back to the user
    CancelDcaOrder {
        initial_asset: AssetInfo,
    },
    /// Creates a new DCA order where `dca_amount` of token `initial_asset` will purchase
    /// `target_asset` every `interval`
    ///
    /// If `initial_asset` is a Cw20 token, the user needs to have increased the allowance prior to
    /// calling this execution
    CreateDcaOrder {
        start_at: u64,
        interval: u64,
        dca_amount: Asset,
        max_hops: Option<u32>,
        max_spread: Option<Decimal>,
        deposit: Asset,
        tip: Asset,
        gas: Asset,
        target_info: AssetInfo,
    },
    /// Modifies an existing DCA order, allowing the user to change certain parameters
    ModifyDcaOrder {
        old_initial_asset: AssetInfo,
        new_initial_asset: Asset,
        new_target_asset: AssetInfo,
        new_interval: u64,
        new_dca_amount: Uint128,
        should_reset_purchase_time: bool,
    },
    /// Performs a DCA purchase for a specified user given a hop route
    PerformDcaPurchase {
        dca_order_id: String,
        hops: Vec<SwapOperation>,
    },

    /// Updates the configuration of the contract
    UpdateConfig {
        /// The new maximum amount of hops to perform from `initial_asset` to `target_asset` when
        /// performing DCA purchases if the user does not specify a custom max hop amount
        max_hops: Option<u32>,
        /// The new fee a user must pay per hop performed in a DCA purchase
        per_hop_fee: Option<Uint128>,
        /// The new whitelisted tokens that can be used in a DCA hop route
        whitelisted_tokens: Option<Vec<AssetInfo>>,
        /// The new maximum spread for DCA purchases
        max_spread: Option<Decimal>,
    },

    /*
    /// Update the configuration for a user
    UpdateUserConfig {
        /// The maximum amount of hops per swap
        max_hops: Option<u32>,
        /// The maximum spread per token when performing DCA purchases
        max_spread: Option<Decimal>,
    },
    */
    /// Withdraws a users bot tip from the contract.
    Withdraw {
        tip: Uint128,
    },

    // Deposit assets into the DCA contract.
    // i) if the asset is native, it is the responsibility of the UI to generate the  deposit tx to the DCA contract
    // ii) if the asset is a token contract, the DCA contract will execute the deposit tx.
    //     However we assume the UI will generate the allowance TX
    Deposit {
        assets: Vec<Asset>,
    },
}

/// This structure describes the query messages available in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns information about the users current active DCA orders in a [`Vec<DcaInfo>`] object.
    UserDcaOrders { user: String },
    /// Returns information about the contract configuration in a [`Config`] object.
    Config {},
    // Returns the users current configuration as a [`UserConfig`] object.
    //  UserConfig { user: String },
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// Describes information for a UserDcaOrders query
///
/// Contains both the user DCA order and the cw20 token allowance, or, if the initial asset is a
/// native token, the balance.
///
/// This is useful for bots and front-end to distinguish between a users token allowance (which may
/// have changed) for the DCA contract, and the created DCA order size.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DcaQueryInfo {
    pub token_allowance: Uint128,
    pub info: DcaInfo,
}
