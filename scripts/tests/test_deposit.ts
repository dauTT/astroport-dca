import { strictEqual } from "assert";
import {
  writeArtifact,
  TokenAsset,
  performTransactionsDebug,
} from "../helpers.js";
import {
  initTestClient,
  TestAccount,
  checkAddressAssetsBalances,
  getDcaOrderId,
} from "./common.js";

import { MsgExecuteContract } from "@terra-money/terra.js";

import { logToFile } from "../util.js";

export async function test_deposit_source_asset() {
  let testName = "test_deposit_source_asset";
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

      let order = await getDcaOrderId(
        terra,
        dca_order_id,
        network,
        logPath,
        "BEFORE DEPOSIT: checking source asset amount"
      );

      let sourceAssetBefore = new TokenAsset(
        network.tokenAddresses.AAA,
        "7800000"
      );

      strictEqual(
        order.balance.source.info.token.contract_addr,
        sourceAssetBefore.getAsset().info.token.contract_addr,
        "Source asset info does not match"
      );

      strictEqual(
        order.balance.source.amount,
        sourceAssetBefore.getAsset().amount,
        "Source asset amount does not match"
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        [new TokenAsset(network.tokenAddresses.AAA, "999990000000")],
        `BEFORE DEPOSIT: balance of the source asset of user=${testAccountAddress}`
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        network.DcaAddress,
        [new TokenAsset(network.tokenAddresses.AAA, "8800000")],
        `BEFORE DEPOSIT: balance of the source asset of DCA contract address=${network.DcaAddress}`
      );

      let deposit_source_asset = new TokenAsset(
        network.tokenAddresses.AAA,
        "200000"
      );

      let msgDeposit = {
        deposit: {
          asset: deposit_source_asset.getAsset(),
          dca_order_id: dca_order_id,
          deposit_type: "source",
        },
      };

      // msg allowance source: contract = network.tokenAddresses.AAA
      let msgIncreaseAllowanceSource = {
        increase_allowance: {
          spender: network.DcaAddress,
          amount: "200000",
        },
      };

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
          msgDeposit,
          []
        ),
      ];

      await performTransactionsDebug(terra, wallet, msgs, logPath);

      order = await getDcaOrderId(
        terra,
        dca_order_id,
        network,
        logPath,
        "AFTER DEPOSIT: checking source asset amount"
      );

      let sourceAssetAfter = new TokenAsset(
        network.tokenAddresses.AAA,
        "8000000"
      );

      strictEqual(
        order.balance.source.info.token.contract_addr,
        sourceAssetAfter.getAsset().info.token.contract_addr,
        "Source asset info does not match"
      );

      strictEqual(
        order.balance.source.amount,
        sourceAssetAfter.getAsset().amount,
        "Source asset amount does not match"
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        testAccountAddress,
        [new TokenAsset(network.tokenAddresses.AAA, "999989800000")],
        `AFTER DEPOSIT: balance of the source asset of user=${testAccountAddress}`
      );

      await checkAddressAssetsBalances(
        terra,
        logPath,
        network.DcaAddress,
        [new TokenAsset(network.tokenAddresses.AAA, "9000000")],
        `AFTER DEPOSIT: balance of the source asset of DCA contract address=${network.DcaAddress}`
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
