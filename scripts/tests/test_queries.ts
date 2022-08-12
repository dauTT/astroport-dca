import { strictEqual, equal } from "assert";
import {
  writeArtifact,
  queryContractDebug,
  TokenAsset,
  NativeAsset,
} from "../helpers.js";
import { getDcaConfig, initTestClient } from "./common.js";
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

    logToFile(
      logPath,
      JSON.stringify(res, null, 4),
      "********** res ******** "
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
      let res = await getDcaConfig(terra, network.DcaAddress, logPath);
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
