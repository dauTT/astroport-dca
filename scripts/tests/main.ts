import {
  test_create_order_1,
  test_create_order_2,
} from "./test_create_order.js";
import {
  test_perform_dca_purchase_for_order_1,
  test_perform_dca_purchase_for_order_2,
} from "./test_perform_dca_purchase.js";
import {
  test_query_invalid_dca_order_id_2,
  test_query_get_config,
  test_query_get_user_dca_orders,
} from "./test_queries.js";
import {
  test_modify_dca_order_id_1,
  test_modify_dca_order_id_1_again,
} from "./test_modify_dca_order.js";
import { test_deposit_source_asset } from "./test_deposit.js";
import { test_withdraw_source_asset } from "./test_withdraw.js";
import { test_cancel_dca_order_2 } from "./test_cancel_dca_order.js";

// Tests summary will be read/written from/to network.json file which is located
// here: astroport_artifacts/localterra.json
// After running a test we write the result in the localterra.json.
// A test can be run only once.
// If one wants to run them again one needs to restore blockchain state to his the initial setup:
//   i) cd localterra
//   ii) local_terra_image.sh rm
//   iii) local_terra_image.sh run
//   iiv) from the working directory 'scripts' execute:
//        node --loader ts-node/esm tests/main.ts
async function main() {
  console.log(
    "***************** Testing DCA contract ************************"
  );

  // The following tests need to be executed in order as some of them have dependencies with the prior tests
  // If one test fail, most likely the successive will all fail.

  await test_create_order_1();
  await test_perform_dca_purchase_for_order_1();
  await test_query_invalid_dca_order_id_2();
  await test_create_order_2();
  await test_perform_dca_purchase_for_order_2();
  await test_query_get_user_dca_orders();
  await test_modify_dca_order_id_1();
  await test_modify_dca_order_id_1_again();
  await test_deposit_source_asset();
  await test_withdraw_source_asset();
  await test_cancel_dca_order_2();
  await test_query_get_config();
}

main().catch(console.log);
