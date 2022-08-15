import { strictEqual } from "assert";
import {
  newClient,
  newTestClient,
  writeArtifact,
  readArtifact,
  deployContract,
  executeContract,
  queryContract,
  executeContractDebug,
  queryContractDebug,
  queryBankDebug,
  toEncodedBinary,
  performTransactions,
  NativeAsset,
  TokenAsset,
  Asset,
} from "../helpers.js";

import { LCDClient, Wallet } from "@terra-money/terra.js";
import { getTokenBalance } from "./snippet.js";

import {
  logToFile,
  deleteFile,
  LOGS_PATH,
  LOCAL_TERRA_TEST_ACCOUNTS,
} from "../util.js";

// One of the test address in LOCAL_TERRA_TEST_ACCOUNTS
export type TestAccount =
  | "test0"
  | "test1"
  | "test2"
  | "test3"
  | "test4"
  | "test5"
  | "test6"
  | "test7"
  | "test8"
  | "test9"
  | "test10";

export interface TestClient {
  wallet: Wallet;
  terra: LCDClient;
  network: any;
  logPath: string;
}

export function initTestClient(
  testName: String,
  testAccount: TestAccount
): TestClient {
  let logPath = `${LOGS_PATH}/${testName}.log`;
  deleteFile(logPath);

  const { terra, wallet } = newTestClient(testAccount);
  strictEqual(
    wallet.key.accAddress,
    LOCAL_TERRA_TEST_ACCOUNTS[testAccount].addr,
    `testAccount=${testAccount} address does not match!`
  );
  logToFile(
    logPath,
    `chainID: ${terra.config.chainID}, user ${testAccount} wallet: ${wallet.key.accAddress}
   `
  );

  // Tests summary will be read/written here in this json object: network
  // Currently this json is located here: astroport_artifacts/localterra.json
  // After running a test we write the result in the json.
  // A test can be run only once.
  // If one wants to run them again one need to restore to the initial setup:
  //     i) Manually remove the tests portion of the localterra.json
  //     ii) Remove the running docker container and re-run a fresh one again
  const network = readArtifact(terra.config.chainID);
  logToFile(
    logPath,
    JSON.stringify(network, null, 4),
    "************ network: ****************"
  );

  return { terra, wallet, network, logPath };
}

export async function getDcaConfig(
  terra: LCDClient,
  network: any,
  logPath: string
): Promise<any> {
  let queryName = "config dca";
  let query = {
    config: {},
  };

  let res = await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );
  return res;
}

export async function checkDcaOrderBalance(
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
}

export async function checkUserAssetBalance(
  terra: LCDClient,
  logPath: string,
  testAccount: string,
  assets: Asset[],
  queryName: string
) {
  logToFile(logPath, "", queryName);
  let userAddress = LOCAL_TERRA_TEST_ACCOUNTS[testAccount].addr;
  assets.forEach(async (asset) => {
    let expected_amount = asset.getAsset().amount;
    if (asset.getInfo().hasOwnProperty("token")) {
      let token_addr = asset.getInfo().token.contract_addr;
      let res = await getTokenBalance(
        terra,
        asset.getInfo().token.contract_addr,
        userAddress,
        logPath
      );
      strictEqual(
        res.balance,
        asset.getAsset().amount,
        `Balance of token=${token_addr}: actual=${res.balance}, expected=${expected_amount}`
      );
    } else {
      let res = await queryBankDebug(
        terra,
        userAddress,
        " natives balance",
        logPath
      );

      console.log("XXXXXXXXXXXXXXXX: ", res[0]._coins["uluna"].amount);
      console.log("XXXXXXXXXXXXXXXX: ", res[0]._coins["uluna"].amount);
      // todo: treat the general case where there a more coins than luna
      let denom = asset.getInfo().native_token.denom;
      let actual_amount = res[0]._coins[denom].amount;
      strictEqual(
        res[0]._coins[denom].amount.toString(),
        expected_amount,
        `Balance of ${denom}: actual=${actual_amount}, expected=${expected_amount}`
      );
    }
  });
}
