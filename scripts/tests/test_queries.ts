import { strictEqual } from "assert";
import {
  writeArtifact,
  queryContractDebug,
  TokenAsset,
  NativeAsset,
} from "../helpers.js";
import { initTestClient, getDcaConfig } from "./common.js";
import { logToFile, LOCAL_TERRA_TEST_ACCOUNTS } from "../util.js";

export async function test_query_invalid_dca_order_id_2() {
  let testName = "test_query_invalid_dca_order_id_2";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test2");

  try {
    let queryName = "Querying for dca_order_id = 2 should fail";
    let query = { dca_orders: { id: "2" } };

    let res = await queryContractDebug(
      terra,
      network.DcaAddress,
      query,
      queryName,
      logPath
    );

    if (!network.tests.test_invalid_dca_order_id) {
      network.tests.test_invalid_dca_order_id = "fail";
    }
  } catch (err) {
    console.error(err);
    logToFile(
      logPath,
      String(err) + ": " + JSON.stringify(err, null, 4),
      "*********** something bad happened: error **************"
    );
    network.tests.test_invalid_dca_order_id = "pass";
  }
  writeArtifact(network, terra.config.chainID);
}

export async function test_query_get_config() {
  let testName = "test_query_get_config";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test2");
  try {
    if (!network.tests.test_query_get_config) {
      let res = await getDcaConfig(terra, network, logPath);
      console.log(res);

      strictEqual(res.owner, LOCAL_TERRA_TEST_ACCOUNTS.test1.addr, "res.owner");
      strictEqual(res.max_hops, 3, "res.max_hops");
      strictEqual(
        res.gas_info.native_token.denom,
        "uluna",
        "res.gas_info.native_token.denom"
      );

      let expectedWhitelistedToken = {
        source: [
          new TokenAsset(network.tokenAddresses.AAA).getInfo(),
          new TokenAsset(network.tokenAddresses.BBB).getInfo(),
          new TokenAsset(network.tokenAddresses.BBB).getInfo(),
          // new TokenAsset(network.tokenAddresses.DDD).getInfo(),
          new NativeAsset("uluna").getInfo(),
        ],
        tip: [
          new TokenAsset(network.tokenAddresses.CCC).getInfo(),
          new TokenAsset(network.tokenAddresses.AAA).getInfo(),
          new NativeAsset("uluna").getInfo(),
        ],
      };
      strictEqual(
        res.whitelisted_tokens.source.toString(),
        expectedWhitelistedToken.source.toString(),
        "res.whitelisted_tokens.source"
      );
      strictEqual(
        res.whitelisted_tokens.tip.toString(),
        expectedWhitelistedToken.tip.toString(),
        "res.whitelisted_tokens.tip"
      );

      strictEqual(res.factory_addr, network.factoryAddress, "res.factory_addr");
      strictEqual(res.router_addr, network.routerAddress, "res.routerAddress");

      network.tests.test_query_get_config = "pass";
    }
  } catch (err) {
    console.error(err);
    logToFile(
      logPath,
      String(err) + ": " + JSON.stringify(err, null, 4),
      "*********** something bad happened: error **************"
    );
    network.tests.test_query_get_config = "fail";
  }
  writeArtifact(network, terra.config.chainID);
}

export async function test_query_get_user_dca_orders() {
  let testName = "test_query_get_user_dca_orders";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test1");

  try {
    if (!network.tests.test_query_get_user_dca_orders) {
      let queryName = `Query all the dca orders of test1 user: ${wallet.key.accAddress}`;
      let query = { user_dca_orders: { user: wallet.key.accAddress } };

      let res = await queryContractDebug(
        terra,
        network.DcaAddress,
        query,
        queryName,
        logPath
      );
      strictEqual(
        res[0],
        "1",
        "It should match with the dca_order_id=1, which was created in test_create_order_1. Make sure to execute the tests in order via the main.ts file"
      );
      strictEqual(
        res[1],
        "2",
        "It should match with the dca_order_id=2, which was created in test_create_order_2. Make sure to execute the tests in order via the main.ts file"
      );
      network.tests.test_query_get_user_dca_orders = "pass";
    }
  } catch (err) {
    console.error(err);
    logToFile(
      logPath,
      String(err) + ": " + JSON.stringify(err, null, 4),
      "*********** something bad happened: error **************"
    );
    network.tests.test_query_get_user_dca_orders = "fail";
  }
  writeArtifact(network, terra.config.chainID);
}
