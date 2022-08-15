// https://github.com/nodejs/node/blob/v17.0.0/lib/assert.js
import { strictEqual } from "assert";

import "dotenv/config";
import {
  newClient,
  newTestClient,
  writeArtifact,
  readArtifact,
  deployContract,
  executeContract,
  queryContract,
  queryContractDebug,
  queryBankDebug,
  toEncodedBinary,
  performTransactions,
  NativeAsset,
  TokenAsset,
  getBlockTimeInSeconds,
} from "../helpers.js";

import { logToFile } from "../util.js";

import {
  Coin,
  Coins,
  isTxError,
  LCDClient,
  LocalTerra,
  MnemonicKey,
  Msg,
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgMigrateContract,
  MsgStoreCode,
  MsgUpdateContractAdmin,
  Tx,
  Wallet,
} from "@terra-money/terra.js";

import { initTestClient } from "./common.js";

type Tests = {
  [key: string]: string;
};

export async function test_create_order_1() {
  let testName = "test_create_order_1";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test1");

  if (!network.tests) {
    let tests: Tests = {};
    network.tests = tests;
  }

  // A test can be run only once.
  if (!network.tests[testName]) {
    try {
      let blocktime = await getBlockTimeInSeconds(terra);
      let queryName: String;
      let query: any;

      //BEFORE: query balance
      queryName = `BEFORE: balance AAA token of the owner (sender): ${wallet.key.accAddress} `;
      query = {
        balance: {
          address: wallet.key.accAddress,
        },
      };
      let res = await queryContractDebug(
        terra,
        network.tokenAddresses.AAA,
        query,
        queryName,
        logPath
      );

      strictEqual(res.balance, "1000000000000", queryName.toString());

      //BEFORE: query allowance
      queryName = `BEFORE: allowance for DCA contract ${network.DcaAddress} to user AAA token of the owner ${wallet.key.accAddress}`;
      query = {
        allowance: {
          owner: wallet.key.accAddress,
          spender: network.DcaAddress,
        },
      };
      await queryContractDebug(
        terra,
        network.tokenAddresses.AAA,
        query,
        queryName,
        logPath
      );
      strictEqual(
        res.allowance ? res.allowance : "0",
        "0",
        queryName.toString()
      );

      queryName = `BEFORE: balance luna of the DCA contract ${network.DcaAddress} `;
      res = await queryBankDebug(
        terra,
        network.DcaAddress,
        queryName.toString(),
        logPath
      );
      strictEqual(res[0].toString(), "", queryName.toString()); // balance of luna is zero

      // msg allowance source: contract = network.tokenAddresses.AAA
      let msgIncreaseAllowanceSource = {
        increase_allowance: {
          spender: network.DcaAddress,
          amount: "10000000",
        },
      };

      // msg allowace tip: contract = network.tokenAddresses.AAA
      let msgIncreaseAllowanceTip = {
        increase_allowance: {
          spender: network.DcaAddress,
          amount: "10000000",
        },
      };

      // msg create order
      let gas = new NativeAsset("uluna", "1000000");
      let msgCreateDcaOrder = {
        create_dca_order: {
          dca_amount: new TokenAsset(
            network.tokenAddresses.AAA,
            "1000000"
          ).getAsset(),
          source: new TokenAsset(
            network.tokenAddresses.AAA,
            "10000000"
          ).getAsset(),
          gas: gas.getAsset(),
          interval: 10,
          start_at: blocktime + 10,
          target_info: new TokenAsset(network.tokenAddresses.BBB).getInfo(),
          tip: new TokenAsset(
            network.tokenAddresses.CCC,
            "10000000"
          ).getAsset(),
        },
      };

      logToFile(
        logPath,
        JSON.stringify(msgCreateDcaOrder, null, 4),
        "********* msgCreateDcaOrder: *********"
      );

      let msgs = [
        // Allow the dca contract to spend money (Source)
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.tokenAddresses.AAA,
          msgIncreaseAllowanceSource,
          []
        ),
        // Allow the dca contract to spend money (Tip)
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.tokenAddresses.CCC,
          msgIncreaseAllowanceTip,
          []
        ),
        // Create dca order. The Dca smart contract will execute a TransferFrom
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.DcaAddress,
          msgCreateDcaOrder,
          [gas.toCoin()]
        ),
      ];

      let header = "*************** msgs: create_dca_order *************";
      logToFile(logPath, JSON.stringify(msgs, null, 4), header);

      res = await performTransactions(terra, wallet, msgs);

      header = "**************** result tx:  ************";
      logToFile(logPath, JSON.stringify(res, null, 4), header);

      // AFTER: query balance
      queryName = `AFTER: balance AAA token of the owner (sender): ${wallet.key.accAddress} `;
      query = {
        balance: {
          address: wallet.key.accAddress,
        },
      };
      res = await queryContractDebug(
        terra,
        network.tokenAddresses.AAA,
        query,
        queryName,
        logPath
      );
      strictEqual(res.balance, "999990000000");

      queryName = `AFTER: balance AAA token of the DCA contract ${network.DcaAddress} `;
      query = {
        balance: {
          address: network.DcaAddress,
        },
      };
      res = await queryContractDebug(
        terra,
        network.tokenAddresses.AAA,
        query,
        queryName,
        logPath
      );
      strictEqual(res.balance, "10000000", queryName.toString()); // source (10000000)

      //AFTER: query allowance
      queryName = `AFTER: allowance for dca contract ${wallet.key.accAddress} to use AAA token of the owner ${wallet.key.accAddress}`;
      query = {
        allowance: {
          owner: wallet.key.accAddress,
          spender: network.DcaAddress,
        },
      };
      res = await queryContractDebug(
        terra,
        network.tokenAddresses.AAA,
        query,
        queryName,
        logPath
      );
      // The allowance is again zero because the DCA contract has executed a transferFrom
      strictEqual(res.allowance, "0", queryName.toString());

      queryName = `AFTER: balance luna of the DCA contract ${network.DcaAddress} `;
      res = await queryBankDebug(
        terra,
        network.DcaAddress,
        queryName.toString(),
        logPath
      );
      strictEqual(
        res[0]._coins.uluna.toString(),
        "1000000uluna",
        queryName.toString()
      ); // -> transfer of  gas (1000000)

      // Check the order is stored properly
      queryName = "dca_orders with id =1 ";
      query = { dca_orders: { id: "1" } };
      res = await queryContractDebug(
        terra,
        network.DcaAddress,
        query,
        queryName,
        logPath
      );

      strictEqual(res.created_by, wallet.key.accAddress, "res.created_by");
      strictEqual(res.interval, 10, "res.interval");
      strictEqual(
        res.dca_amount.info.token.contract_addr,
        network.tokenAddresses.AAA,
        "res.dca_amount.info.token.contract_addr"
      );
      strictEqual(res.dca_amount.amount, "1000000", "res.dca_amount.amount");
      strictEqual(
        res.balance.source.info.token.contract_addr,
        network.tokenAddresses.AAA,
        "res.balance.source.info.token.contract_addr"
      );
      strictEqual(
        res.balance.source.amount,
        "10000000",
        "res.balance.source.amount"
      );
      strictEqual(
        res.balance.spent.info.token.contract_addr,
        network.tokenAddresses.AAA,
        "res.balance.spent.info.token.contract_addr"
      );
      strictEqual(res.balance.spent.amount, "0", "res.balance.spent.amount");
      strictEqual(
        res.balance.gas.info.native_token.denom,
        "uluna",
        "res.balance.gas.info.native_token.denom"
      );
      strictEqual(res.balance.gas.amount, "1000000", "res.balance.gas.amount");
      strictEqual(
        res.balance.target.info.token.contract_addr,
        network.tokenAddresses.BBB,
        "res.balance.target.info.token.contract_addr"
      );
      strictEqual(res.balance.target.amount, "0", "res.balance.target.amount");
      strictEqual(
        res.balance.tip.info.token.contract_addr,
        network.tokenAddresses.CCC,
        "res.balance.tip.info.token.contract_addr"
      );
      strictEqual(res.balance.tip.amount, "10000000", "res.balance.tip.amount");
      strictEqual(res.balance.last_purchase, 0, "res.balance.last_purchase");
      strictEqual(res.max_hops, null, "res.max_hops");
      strictEqual(res.max_spread, null, "res.max_spread");

      network.tests[testName] = "pass";
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
}

export async function test_create_order_2() {
  let testName = "test_create_order_2";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test1");

  if (!network.tests) {
    let tests: Tests = {};
    network.tests = tests;
  }

  if (!network.tests[testName]) {
    try {
      let blocktime = await getBlockTimeInSeconds(terra);
      let queryName: String;
      let query: any;
      let res: any;

      // msg allowance source: contract = network.tokenAddresses.BBB
      let msgIncreaseAllowanceSource = {
        increase_allowance: {
          spender: network.DcaAddress,
          amount: "8000000",
        },
      };

      // msg create order
      let gas = new NativeAsset("uluna", "1000000");
      let tip = new NativeAsset("uluna", "5000000");
      let gas_and_tip = new NativeAsset("uluna", "6000000");
      let msgCreateDcaOrder = {
        create_dca_order: {
          max_hops: 3,
          max_spread: "0.4",
          dca_amount: new TokenAsset(
            network.tokenAddresses.AAA,
            "200000"
          ).getAsset(),
          source: new TokenAsset(
            network.tokenAddresses.AAA,
            "8000000"
          ).getAsset(),
          gas: gas.getAsset(),
          interval: 10,
          start_at: blocktime + 10,
          target_info: new NativeAsset("uluna").getInfo(),
          tip: tip.getAsset(),
        },
      };

      logToFile(
        logPath,
        JSON.stringify(msgCreateDcaOrder, null, 4),
        "********* msgCreateDcaOrder: *********"
      );

      let msgs = [
        // Allow the dca contract to spend money (Source)
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.tokenAddresses.AAA,
          msgIncreaseAllowanceSource,
          []
        ),
        // Create dca order. The Dca smart contract will execute a TransferFrom
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.DcaAddress,
          msgCreateDcaOrder,
          [gas_and_tip.toCoin()]
        ),
      ];

      logToFile(
        logPath,
        JSON.stringify(msgs, null, 4),
        "*************** msgs: create_dca_order *************"
      );

      res = await performTransactions(terra, wallet, msgs);

      logToFile(
        logPath,
        JSON.stringify(res, null, 4),
        "**************** result tx:  ************"
      );

      // Check the order is stored properly
      queryName = "dca_orders with id =2 ";
      query = { dca_orders: { id: "2" } };
      res = await queryContractDebug(
        terra,
        network.DcaAddress,
        query,
        queryName,
        logPath
      );

      strictEqual(res.max_spread, "0.4", "res.max_spread");
      strictEqual(res.max_hops, 3, "res.max_spread");

      network.tests[testName] = "pass";
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
}

export async function test_create_order_3() {
  let testName = "test_create_order_3";
  const { terra, wallet, network, logPath } = initTestClient(testName, "test1");

  if (!network.tests) {
    let tests: Tests = {};
    network.tests = tests;
  }

  if (!network.tests[testName]) {
    try {
      let blocktime = await getBlockTimeInSeconds(terra);
      let queryName: String;
      let query: any;
      let res: any;

      // msg allowance source: contract = network.tokenAddresses.BBB
      let msgIncreaseAllowanceSource = {
        increase_allowance: {
          spender: network.DcaAddress,
          amount: "8000000",
        },
      };

      // msg create order
      let gas = new NativeAsset("uluna", "1000000");
      let tip = new NativeAsset("uluna", "5000000");
      let gas_and_tip = new NativeAsset("uluna", "6000000");
      let msgCreateDcaOrder = {
        create_dca_order: {
          max_hops: 3,
          max_spread: "0.4",
          dca_amount: new TokenAsset(
            network.tokenAddresses.AAA,
            "200000"
          ).getAsset(),
          source: new TokenAsset(
            network.tokenAddresses.AAA,
            "8000000"
          ).getAsset(),
          gas: gas.getAsset(),
          interval: 10,
          start_at: blocktime + 10,
          target_info: new NativeAsset("uluna").getInfo(),
          tip: new TokenAsset(network.tokenAddresses.AAA, "200000"),
        },
      };

      logToFile(
        logPath,
        JSON.stringify(msgCreateDcaOrder, null, 4),
        "********* msgCreateDcaOrder: *********"
      );

      let msgs = [
        // Allow the dca contract to spend money (Source)
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.tokenAddresses.AAA,
          msgIncreaseAllowanceSource,
          []
        ),
        // Create dca order. The Dca smart contract will execute a TransferFrom
        new MsgExecuteContract(
          wallet.key.accAddress,
          network.DcaAddress,
          msgCreateDcaOrder,
          [gas_and_tip.toCoin()]
        ),
      ];

      logToFile(
        logPath,
        JSON.stringify(msgs, null, 4),
        "*************** msgs: create_dca_order *************"
      );

      res = await performTransactions(terra, wallet, msgs);

      logToFile(
        logPath,
        JSON.stringify(res, null, 4),
        "**************** result tx:  ************"
      );

      // Check the order is stored properly
      queryName = "dca_orders with id =2 ";
      query = { dca_orders: { id: "2" } };
      res = await queryContractDebug(
        terra,
        network.DcaAddress,
        query,
        queryName,
        logPath
      );

      strictEqual(res.max_spread, "0.4", "res.max_spread");
      strictEqual(res.max_hops, 3, "res.max_spread");

      network.tests[testName] = "pass";
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
}
