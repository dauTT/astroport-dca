import { strictEqual } from "assert";

import {
  newClient,
  newTestClient,
  readArtifact,
  queryContractDebug,
} from "../helpers.js";

import { LCDClient, Wallet } from "@terra-money/terra.js";

import {
  logToFile,
  deleteFile,
  LOGS_PATH,
  LOCAL_TERRA_TEST_ACCOUNTS,
} from "../util.js";

// One of the test address in LOCAL_TERRA_TEST_ACCOUNTS
type TestAccount =
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
  dcaAddress: any,
  logPath: string
): Promise<any> {
  let queryName = "config dca";
  let query = {
    config: {},
  };

  let res = await queryContractDebug(
    terra,
    dcaAddress,
    query,
    queryName,
    logPath
  );
  return res;
}
