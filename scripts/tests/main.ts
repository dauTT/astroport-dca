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
import { test_modify_dca_order_id_1 } from "./test_modify_dca_order.js";

// Tests summary will be read/written from/to network json file which is located
// here astroport_artifacts/localterra.json
// After running a test we write the result in the json.
// A test can be run only once.
// If one wants to run them again one needs to restore blockchain state to his the initial setup:
//   i) Manually remove the tests portion of the localterra.json
//   ii) Remove the running docker container and re-run a fresh one again
async function main() {
  console.log(
    "***************** Testing DCA contract ************************"
  );

  await test_create_order_1();
  await test_perform_dca_purchase_for_order_1();
  await test_query_invalid_dca_order_id_2();
  await test_create_order_2();
  await test_perform_dca_purchase_for_order_2();
  //await test_query_get_user_dca_orders();
  // await test_perform_dca_purchase_for_order_2();
  //await test_modify_dca_order_id_1();
  // await test_query_get_config();
}

main().catch(console.log);
