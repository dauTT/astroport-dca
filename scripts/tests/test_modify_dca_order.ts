import { strictEqual } from "assert";
import {
  writeArtifact,
  queryContractDebug,
  TokenAsset,
  NativeAsset,
} from "../helpers.js";
import { getDcaConfig, initTestClient } from "./common.js";
import { logToFile, LOCAL_TERRA_TEST_ACCOUNTS } from "../util.js";

export async function test_modify_dca_order_id_1() {
  let testName = "test_modify_dca_order_id_1";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test1");
  try {
    if (!network.tests.test_modify_dca_order_id_1) {
      let queryName = "Querying for dca_order_id = 1";
      let query = { dca_orders: { id: "1" } };

      let res = await queryContractDebug(
        terra,
        network.DcaAddress,
        query,
        queryName,
        logPath
      );

      // network.tests.test_modify_dca_order_id_1 = "pass";
    }
  } catch (err) {
    console.error(err);
    logToFile(
      logPath,
      String(err) + ": " + JSON.stringify(err, null, 4),
      "*********** something bad happened: error **************"
    );
    // network.tests.test_modify_dca_order_id_1 = "fail";
  }
  writeArtifact(network, terra.config.chainID);
}
