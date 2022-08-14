// https://github.com/nodejs/node/blob/v17.0.0/lib/assert.js
import { strictEqual } from "assert";

import "dotenv/config";
import {
  newClient,
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
} from "../helpers.js";
import { join } from "path";

import { logToFile, deleteFile, LOGS_PATH } from "../util.js";

import { initTestClient } from "./common.js";

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

// This is a kind of playground for designing concrete tests.
// Create small runnable snippet here till you can aggregated them into a test
async function main() {
  const { terra, wallet, network, logPath } = initTestClient(
    "snippet",
    "test1"
  );

  let queryName: String;
  let query: any;

  queryName = "config dca";
  query = {
    config: {},
  };

  let res = await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );
  /*
  queryName = "pair: AAA-BBB";
  query = {
    pair: {
      asset_infos: [
        new TokenAsset(network.tokenAddresses.AAA).getInfo(),
        new TokenAsset(network.tokenAddresses.BBB).getInfo(),
      ],
    },
  };

  res = await queryContractDebug(
    terra,
    network.factoryAddress,
    query,
    queryName,
    logPath
  );

*/

  queryName = "lunabalance dca address";
  await queryBankDebug(terra, network.DcaAddress, queryName, logPath);

  queryName = "dca_orders with id =2 ";
  query = { dca_orders: { id: "2" } };
  await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );

  queryName = "sub_msg ";
  query = { reply_sub_msg_response: {} };
  await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );
}

main().catch(console.log);
