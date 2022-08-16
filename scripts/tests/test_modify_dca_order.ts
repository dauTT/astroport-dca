import {
  writeArtifact,
  TokenAsset,
  NativeAsset,
  executeContractDebug,
  performTransactionsDebug,
} from "../helpers.js";
import {
  initTestClient,
  checkDcaOrderBalance,
  checkAddressAssetsBalances,
  TestAccount,
} from "./common.js";

import { MsgExecuteContract } from "@terra-money/terra.js";

import { logToFile } from "../util.js";

export async function test_modify_dca_order_id_1() {
  let testName = "test_modify_dca_order_id_1";
  let testAccount: TestAccount = "test1";
  const { terra, wallet, network, logPath } = initTestClient(
    testName,
    testAccount
  );
  const testAccountAddress = wallet.key.accAddress;

  try {
    if (!network.tests[testName]) {
      let queryName = "Querying for dca_order_id = 1";
      let dca_order_id = "1";

      let sourceAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "9000000"
      );
      let spentAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "1000000"
      );
      let tagetAssetBefore = new TokenAsset(
        network.tokenAddresses.BBB,
        "996991"
      );
      let gasAssetBefore = new NativeAsset("uluna", "1000000");
      let tipAssetBefore = new TokenAsset(
        network.tokenAddresses.CCC,
        "9900000"
      );

      queryName = `BEFORE MODIFICATION: Checking Balances of dca_order_id=${dca_order_id}`;
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
        queryName
      );

      let assetsBefore = [
        new NativeAsset("uluna", "999999969439803"),
        new TokenAsset(network.tokenAddresses.AAA, "999982000000"),
        new TokenAsset(network.tokenAddresses.BBB, "1000000000000"),
      ];
      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        assetsBefore,
        "BEFORE MODIFICATION: User Assets Balances"
      );

      let new_asset_source = new NativeAsset("uluna", "5000000");
      let msgModifyDcaOrder = {
        modify_dca_order: {
          id: dca_order_id,
          new_source_asset: new_asset_source.getAsset(),
          new_target_asset_info: new TokenAsset(
            network.tokenAddresses.DDD
          ).getInfo(),
          new_dca_amount: new NativeAsset("uluna", "10000").getAsset(),
        },
      };

      logToFile(
        logPath,
        JSON.stringify(msgModifyDcaOrder, null, 4),
        "********* msgModifyDcaOrder: *********"
      );

      await executeContractDebug(
        terra,
        wallet,
        network.DcaAddress,
        msgModifyDcaOrder,
        [new_asset_source.toCoin()],
        `********** execute modify_dca_order: ${dca_order_id}$ ***********`,
        logPath
      );

      let sourceAssetAfter = new NativeAsset("uluna", "5000000");
      let spentAssetAfter = new NativeAsset("uluna", "0");
      let tagetAssetAfter = new TokenAsset(network.tokenAddresses.DDD, "0");
      let gasAssetAfter = new NativeAsset("uluna", "1000000");
      let tipAssetAfter = new TokenAsset(network.tokenAddresses.CCC, "9900000");
      queryName = `AFTER MODIFICATION: Checking Balances of dca_order_id=${dca_order_id}`;
      await checkDcaOrderBalance(
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

      let assetsAfter = [
        new NativeAsset("uluna", "999999964357494"), // 999999969439840 - 5000000 => luna is slightly less because we need to pay also for the modification tx
        new TokenAsset(network.tokenAddresses.AAA, "999991000000"), // 999982000000 -9000000
        new TokenAsset(network.tokenAddresses.BBB, "1000000996991"), // 1000000000000 + 996991
      ];
      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        assetsAfter,
        "AFTER MODIFICATION: User Assets Balances"
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

export async function test_modify_dca_order_id_1_again() {
  let testName = "test_modify_dca_order_id_1_again";
  let testAccount: TestAccount = "test1";
  const { terra, wallet, network, logPath } = initTestClient(
    testName,
    testAccount
  );
  const testAccountAddress = wallet.key.accAddress;

  try {
    if (!network.tests[testName]) {
      let queryName = "Querying for dca_order_id = 1";
      let dca_order_id = "1";

      let sourceAssetBefore = new NativeAsset("uluna", "5000000");
      let spentAssetBefore = new NativeAsset("uluna", "0");
      let tagetAssetBefore = new TokenAsset(network.tokenAddresses.DDD, "0");
      let gasAssetBefore = new NativeAsset("uluna", "1000000");
      let tipAssetBefore = new TokenAsset(
        network.tokenAddresses.CCC,
        "9900000"
      );
      let interval = 10;
      let max_hops = undefined;
      let max_spread = undefined;

      queryName = `BEFORE MODIFICATION: Checking Balances of dca_order_id=${dca_order_id}`;

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
        queryName,
        interval,
        max_hops,
        max_spread
      );

      let assetsBefore = [
        new NativeAsset("uluna", "999999964357494"),
        new TokenAsset(network.tokenAddresses.AAA, "999991000000"),
        new TokenAsset(network.tokenAddresses.BBB, "1000000996991"),
        new TokenAsset(network.tokenAddresses.EEE, "1000000000000"),
        new TokenAsset(network.tokenAddresses.CCC, "999990000000"),
      ];
      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        assetsBefore,
        "BEFORE MODIFICATION: User Assets Balances"
      );

      let new_asset_source = new TokenAsset(
        network.tokenAddresses.AAA,
        "1000000"
      );
      let new_tip_asset = new NativeAsset("uluna", "64357531");
      let new_interval = 1000;
      let new_start_at = 1000;
      let new_max_hops = 5;
      let new_max_spread = "0.7";

      // msg allowance source: contract = network.tokenAddresses.AAA
      let msgIncreaseAllowanceSource = {
        increase_allowance: {
          spender: network.DcaAddress,
          amount: "1000000",
        },
      };

      let msgModifyDcaOrder = {
        modify_dca_order: {
          id: dca_order_id,
          new_source_asset: new_asset_source.getAsset(),
          new_target_asset_info: new TokenAsset(
            network.tokenAddresses.EEE
          ).getInfo(),
          new_dca_amount: new TokenAsset(
            network.tokenAddresses.AAA,
            "500000"
          ).getAsset(), //new NativeAsset("uluna", "10000").getAsset(),
          new_tip_asset: new_tip_asset.getAsset(),
          new_interval: 1000,
          new_start_at: 1000,
          new_max_hops: 5,
          new_max_spread: "0.7",
        },
      };

      logToFile(
        logPath,
        JSON.stringify(msgModifyDcaOrder, null, 4),
        "********* msgModifyDcaOrder: *********"
      );

      let msgs = [
        // Allow the dca contract to spend money (Source)
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.tokenAddresses.AAA,
          msgIncreaseAllowanceSource,
          []
        ),
        // Modify dca order. The Dca smart contract will execute a TransferFrom (move token AAA from user to dca contract)
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.DcaAddress,
          msgModifyDcaOrder,
          [new_tip_asset.toCoin()]
        ),
      ];

      await performTransactionsDebug(terra, wallet, msgs, logPath);

      let sourceAssetAfter = new_asset_source;
      let spentAssetAfter = new TokenAsset(network.tokenAddresses.AAA, "0");
      let tagetAssetAfter = new TokenAsset(network.tokenAddresses.EEE, "0");
      let gasAssetAfter = new NativeAsset("uluna", "1000000");
      let tipAssetAfter = new NativeAsset("uluna", "64357531");
      queryName = `AFTER MODIFICATION: Checking Balances of dca_order_id=${dca_order_id}`;

      await checkDcaOrderBalance(
        terra,
        logPath,
        network.DcaAddress,
        sourceAssetAfter,
        spentAssetAfter,
        tagetAssetAfter,
        gasAssetAfter,
        tipAssetAfter,
        dca_order_id,
        queryName.toString(),
        new_interval,
        new_start_at,
        new_max_hops,
        new_max_spread
      );

      let assetsAfter = [
        new NativeAsset("uluna", "999999904872392"), // 999999964357494 - 64357531 + 5000000=> luna is slightly less because we need to pay also for the modification tx
        new TokenAsset(network.tokenAddresses.AAA, "999990000000"), // 999991000000 -1000000
        new TokenAsset(network.tokenAddresses.BBB, "1000000996991"), // Nothing has changed
        new TokenAsset(network.tokenAddresses.EEE, "1000000000000"), // Nothing has changed
        new TokenAsset(network.tokenAddresses.CCC, "999999900000"), // 999990000000 + 9900000
      ];
      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        assetsAfter,
        "AFTER MODIFICATION: User Assets Balances"
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
