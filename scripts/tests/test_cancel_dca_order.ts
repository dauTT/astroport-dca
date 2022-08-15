import { strictEqual } from "assert";
import {
  writeArtifact,
  TokenAsset,
  NativeAsset,
  performTransactionsDebug,
} from "../helpers.js";
import {
  initTestClient,
  TestAccount,
  checkAddressAssetsBalances,
  getDcaOrderId,
  checkDcaOrderBalance,
  checkUserOrders,
} from "./common.js";

import { MsgExecuteContract } from "@terra-money/terra.js";

import { logToFile } from "../util.js";

export async function test_cancel_dca_order_2() {
  let testName = "test_cancel_dca_order_2";
  let testAccount: TestAccount = "test1";
  const { terra, wallet, network, logPath } = initTestClient(
    testName,
    testAccount
  );
  const testAccountAddress = wallet.key.accAddress;

  try {
    if (!network.tests[testName]) {
      let queryName = "Querying for dca_order_id = 2";
      let dca_order_id = "2";

      let sourceAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "7000000"
      );
      let spentAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "200000"
      );
      let tagetAssetBefore = new NativeAsset("uluna", "199297");
      let gasAssetBefore = new NativeAsset("uluna", "1000000");
      let tipAssetBefore = new NativeAsset("uluna", "4800000");

      await checkDcaOrderBalance(
        terra,
        logPath,
        network.DcaAddress,
        sourceAssetBefore,
        spentAssetBefore,
        tagetAssetBefore,
        gasAssetBefore,
        tipAssetBefore,
        dca_order_id,
        `BEFOR CANCEL ODER: Checking Balances of dca_order_id=${dca_order_id}`
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        [
          new TokenAsset(network.tokenAddresses.AAA, "999990800000"),
          new NativeAsset("uluna", "999999904733468"),
        ],
        `BEFOR CANCEL ODER: Check balances of Token AAA and luna of user=${testAccountAddress}`
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        network.DcaAddress,
        [
          new TokenAsset(network.tokenAddresses.AAA, "8000000"),
          new NativeAsset("uluna", "71356828"),
        ],
        `BEFOR CANCEL ODER: Check balances of Token AAA and luna of DCA contract address=${network.DcaAddress}`
      );

      // Cancel Dca order
      let msgCancelDcaOder = {
        cancel_dca_order: {
          id: dca_order_id,
        },
      };

      let msgs = [
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.DcaAddress,
          msgCancelDcaOder,
          []
        ),
      ];

      await performTransactionsDebug(terra, wallet, msgs, logPath);

      await checkUserOrders(
        terra,
        testAccountAddress,
        network,
        ["1"],
        `AFTER CANCEL ODER: Check user dca order ids`,
        logPath
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        [
          new TokenAsset(network.tokenAddresses.AAA, "999997800000"),
          new NativeAsset("uluna", "999999910670187"), // 999999904733468 + (tip) 4800000 + (gas) 1000000  + (target) 199297 => does not match exactly due to gas fee
        ],
        `AFTER CANCEL ODER: Check balances of Token AAA and luna of user=${testAccountAddress}`
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        network.DcaAddress,
        [
          new TokenAsset(network.tokenAddresses.AAA, "1000000"),
          new NativeAsset("uluna", "65357531"), // 71356828 - (tip) 4800000 - (gas) 1000000  - (target) 199297
        ],
        `AFTER CANCEL ORDER: Check balances of Token AAA and luna of DCA contract address=${network.DcaAddress}`
      );

      network.tests[testName] = "pass";
    }
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
