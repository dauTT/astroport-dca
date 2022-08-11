// https://github.com/nodejs/node/blob/v17.0.0/lib/assert.js
import { strictEqual } from "assert";

import "dotenv/config";

import { test_create_order_1_hop } from "./test_create_order.js";

async function main() {
  console.log(
    "***************** Testing DCA contract ************************"
  );
  await test_create_order_1_hop();
  // await test_invalid_dca_order_id(terra, wallet, network);
  // await test_perform_dca_purchase(terra, wallet, network);
  // await test_create_order_2(terra, wallet, network);
}

main().catch(console.log);
