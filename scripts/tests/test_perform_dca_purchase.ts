import { strictEqual } from "assert";
import {
  newClient,
  writeArtifact,
  readArtifact,
  deployContract,
  executeContract,
  queryContract,
  executeContractDebug,
  queryContractDebug,
  toEncodedBinary,
  performTransactions,
  NativeAsset,
  TokenAsset,
  Asset,
  newTestClient,
} from "../helpers.js";
import { join } from "path";

import { getDcaConfig } from "./common.js";
import * as fs from "fs";

import {
  Coin,
  Coins,
  isTxError,
  LCDClient,
  LocalTerra,
  MnemonicKey,
  Msg,
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgMigrateContract,
  MsgStoreCode,
  MsgUpdateContractAdmin,
  Tx,
  Wallet,
} from "@terra-money/terra.js";

import { initTestClient } from "./common.js";
import { logToFile } from "../util.js";

// Perform_dca_purchase for the order created in test_create_order_1_hop
export async function test_perform_dca_purchase_for_order_1() {
  let testName = "test_perform_dca_purchase_for_order_1";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test4");

  if (!network.tests[testName]) {
    try {
      // let dcaConfig = await getDcaConfig(terra, network, logPath);
      let dca_order_id = "1";
      let queryName: string;

      let sourceAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "10000000"
      );
      let spentAssetBefore = new TokenAsset(network.tokenAddresses.AAA, "0");
      let tagetAssetBefore = new TokenAsset(network.tokenAddresses.BBB, "0");
      let gasAssetBefore = new NativeAsset("uluna", "1000000");
      let tipAssetBefore = new TokenAsset(
        network.tokenAddresses.CCC,
        "10000000"
      );

      queryName = `BEFORE PURCHASE: Checking Balances of dca_order_id=${dca_order_id}`;
      await checkBalance(
        terra,
        logPath,
        network.DcaAddress,
        sourceAssetBefore,
        spentAssetBefore,
        tagetAssetBefore,
        gasAssetBefore,
        tipAssetBefore,
        dca_order_id,
        queryName
      );

      queryName = "BEFORE PURCHASE: Check pool AAA-BBB";
      await checkTokenPool(
        terra,
        network,
        logPath,
        "poolAAA-BBB",
        "AAA",
        "BBB",
        "100000000000",
        "100000000000",
        `BEFORE PURCHASE: Check pool AAA-BBB`
      );

      let msg = {
        perform_dca_purchase: {
          dca_order_id: dca_order_id,
          hops: [
            // AAA -> BBB
            {
              astro_swap: {
                offer_asset_info: {
                  token: {
                    contract_addr: network.tokenAddresses.AAA,
                  },
                },
                ask_asset_info: {
                  token: {
                    contract_addr: network.tokenAddresses.BBB,
                  },
                },
              },
            },
          ],
        },
      };

      await executeContractDebug(
        terra,
        wallet,
        network.DcaAddress,
        msg,
        [],
        "********** perform_dca_purchase ***********",
        logPath
      );

      await checkTokenPool(
        terra,
        network,
        logPath,
        "poolAAA-BBB",
        "AAA",
        "BBB",
        "100001000000",
        "99999002010",
        `AFTER PURCHASE: Check pool AAA-BBB`
      );

      let sourceAssetAfter = new TokenAsset(
        network.tokenAddresses.AAA,
        "9000000"
      );
      let spentAssetAfter = new TokenAsset(
        network.tokenAddresses.AAA,
        "1000000"
      );
      let tagetAssetAfter = new TokenAsset(
        network.tokenAddresses.BBB,
        "996991"
      );
      let gasAssetAfter = new NativeAsset("uluna", "1000000");
      let tipAssetAfter = new TokenAsset(network.tokenAddresses.CCC, "9900000");
      queryName = `AFTER PURCHASE: Checking Balances of dca_order_id=${dca_order_id}`;

      await checkBalance(
        terra,
        logPath,
        network.DcaAddress,
        sourceAssetAfter,
        spentAssetAfter,
        tagetAssetAfter,
        gasAssetAfter,
        tipAssetAfter,
        dca_order_id,
        queryName.toString()
      );

      network.tests[testName] = "pass";
    } catch (err) {
      console.error(err);
      logToFile(
        logPath,
        String(err) + ": " + JSON.stringify(err, null, 4),
        "*********** something bad happened: error **************"
      );
      network.tests[testName] = "fail";
    }

    writeArtifact(network, terra.config.chainID);
  }
}

// Perform_dca_purchase for the order created in test_create_order_2
export async function test_perform_dca_purchase_for_order_2() {
  let testName = "test_perform_dca_purchase_for_order_2";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test4");
  if (!network.tests[testName]) {
    try {
      // let dcaConfig = await getDcaConfig(terra, network, logPath);
      let dca_order_id = "2";
      let queryName: string;

      let sourceAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "8000000"
      );
      let spentAssetBefore = new TokenAsset(network.tokenAddresses.AAA, "0");
      let tagetAssetBefore = new NativeAsset("uluna", "0");
      let gasAssetBefore = new NativeAsset("uluna", "1000000");
      let tipAssetBefore = new NativeAsset("uluna", "5000000");

      queryName = `BEFORE PURCHASE: Checking Balances of dca_order_id=${dca_order_id}`;
      await checkBalance(
        terra,
        logPath,
        network.DcaAddress,
        sourceAssetBefore,
        spentAssetBefore,
        tagetAssetBefore,
        gasAssetBefore,
        tipAssetBefore,
        dca_order_id,
        queryName
      );

      queryName = "BEFORE PURCHASE: Check pool AAA-BBB";
      await checkTokenPool(
        terra,
        network,
        logPath,
        "poolAAA-BBB",
        "AAA",
        "BBB",
        "100001000000",
        "99999002010",
        `BEFORE PURCHASE: Check pool AAA-BBB`
      );

      let msg = {
        perform_dca_purchase: {
          dca_order_id: dca_order_id,
          hops: [
            // AAA -> BBB ->LUNA
            {
              // AAA -> BBB
              astro_swap: {
                offer_asset_info: {
                  token: {
                    contract_addr: network.tokenAddresses.AAA,
                  },
                },
                ask_asset_info: {
                  token: {
                    contract_addr: network.tokenAddresses.BBB,
                  },
                },
              },
            },
            {
              // BBB ->LUNA
              astro_swap: {
                offer_asset_info: {
                  token: {
                    contract_addr: network.tokenAddresses.BBB,
                  },
                },
                ask_asset_info: {
                  native_token: {
                    denom: "uluna",
                  },
                },
              },
            },
          ],
        },
      };

      await executeContractDebug(
        terra,
        wallet,
        network.DcaAddress,
        msg,
        [],
        "********** perform_dca_purchase ***********",
        logPath
      );

      await checkTokenPool(
        terra,
        network,
        logPath,
        "poolAAA-BBB",
        "AAA",
        "BBB",
        "100001200000",
        "99998802415",
        `AFTER PURCHASE: Check pool AAA-BBB`
      );

      let sourceAssetAfter = new TokenAsset(
        network.tokenAddresses.AAA,
        "7800000"
      );
      let spentAssetAfter = new TokenAsset(
        network.tokenAddresses.AAA,
        "200000"
      );
      let tagetAssetAfter = new NativeAsset("uluna", "199297");

      // todo: why is the gas still 1000000? I would have expected it to be slighly less.
      let gasAssetAfter = new NativeAsset("uluna", "1000000");

      // per_hop_fee=100000, hops=2 =>  tip_cost = per_hop_fee * hops
      // => tip_balance_before = 5000000
      // => tip_balance_afer = tip_balance_before - tip_cost = 4800000
      let tipAssetAfter = new NativeAsset("uluna", "4800000");
      queryName = `AFTER PURCHASE: Checking Balances of dca_order_id=${dca_order_id}`;

      await checkBalance(
        terra,
        logPath,
        network.DcaAddress,
        sourceAssetAfter,
        spentAssetAfter,
        tagetAssetAfter,
        gasAssetAfter,
        tipAssetAfter,
        dca_order_id,
        queryName.toString()
      );

      network.tests[testName] = "pass";
    } catch (err) {
      console.error(err);
      logToFile(
        logPath,
        String(err) + ": " + JSON.stringify(err, null, 4),
        "*********** something bad happened: error **************"
      );
      network.tests[testName] = "fail";
    }

    writeArtifact(network, terra.config.chainID);
  }
}

async function checkBalance(
  terra: LCDClient,
  logPath: string,
  DcaAddress: string,
  source: Asset,
  spent: Asset,
  target: Asset,
  gas: Asset,
  tip: Asset,
  dca_order_id: string,
  queryName: string
) {
  let keys = ["source", "spent", "target", "gas", "tip"];
  let values = [
    source.getAsset(),
    spent.getAsset(),
    target.getAsset(),
    gas.getAsset(),
    tip.getAsset(),
  ];

  let expectedBalance: { [key: string]: any } = {};
  keys.forEach((key, i) => (expectedBalance[key] = values[i]));

  let query = {
    dca_orders: { id: dca_order_id },
  };

  let res = await queryContractDebug(
    terra,
    DcaAddress,
    query,
    queryName,
    logPath
  );

  keys.forEach((key) => {
    if (expectedBalance[key].info.hasOwnProperty("token")) {
      strictEqual(
        res.balance[key].info.token.contract_addr,
        expectedBalance[key].info.token.contract_addr,
        `Check ${key} address`
      );
    } else {
      strictEqual(
        res.balance[key].info.native_token.denom,
        expectedBalance[key].info.native_token.denom,
        `Check ${key} denom`
      );
    }

    strictEqual(
      res.balance[key].amount,
      expectedBalance[key].amount,
      `Check ${key} amount`
    );
  });

  // let tip_cost = Number(dcaConfig.per_hop_fee) * msg.perform_dca_purchase.hops.length
  // strictEqual(res.balance.tip.amount, "9900000", "Check gas amount");
}

async function checkTokenPool(
  terra: LCDClient,
  network: any,
  logPath: string,
  poolAddress: string,
  asset1: string,
  asset2: string,
  asset1Amount: string,
  asset2Amount: string,
  queryName: string
) {
  let query = {
    pool: {},
  };
  let res = await queryContractDebug(
    terra,
    network[poolAddress],
    query,
    queryName,
    logPath
  );

  strictEqual(
    res.assets[0].info.token.contract_addr,
    network.tokenAddresses[asset1],
    "Check asset1 address"
  );
  strictEqual(res.assets[0].amount, asset1Amount, "Check asset1 amount");
  strictEqual(
    res.assets[1].info.token.contract_addr,
    network.tokenAddresses[asset2],
    "Check asset2 address"
  );
  strictEqual(res.assets[1].amount, asset2Amount, "Check asset2 amount");
}
