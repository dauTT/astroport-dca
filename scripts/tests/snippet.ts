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
import * as fs from "fs";

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

import { LOCAL_TERRA_TEST_ACCOUNTS } from "../util.js";

// This is a kind of playground for designing concrete tests.
// Create small runnable snippet here till you can aggregated them into a test
async function main() {
  const { terra, wallet, network, logPath } = initTestClient(
    "snippet",
    "test1"
  );

  await queryBankDebug(terra, network.DcaAddress, " luna balance", logPath);
  await get_token_balance(
    terra,
    network.tokenAddresses.AAA,
    LOCAL_TERRA_TEST_ACCOUNTS["test1"].addr,
    logPath
  );
  /*
  await get_dca_config(terra, logPath, network);
  await get_dca_order_id(terra, "1", logPath, network);
  await get_reply_sub_msg(terra, logPath, network);

  await get_dca_config(terra, logPath, network);
  await query_pool(
    terra,
    new TokenAsset(network.tokenAddresses.AAA).getInfo(),
    new TokenAsset(network.tokenAddresses.BBB).getInfo(),
    logPath,
    network
  );
*/
}

main().catch(console.log);

async function query_pool(
  terra: LCDClient,
  asset1_info: any,
  asset2_info: any,
  logPath: fs.PathOrFileDescriptor,
  network: any
): Promise<any> {
  let queryName = "Query pool";
  let query = {
    pair: {
      asset_infos: [asset1_info, asset2_info],
    },
  };

  return await queryContractDebug(
    terra,
    network.factoryAddress,
    query,
    queryName,
    logPath
  );
}

async function get_token_balance(
  terra: LCDClient,
  contract_addr: string,
  user_addr: string,
  logPath: fs.PathOrFileDescriptor
): Promise<any> {
  let queryName = `Get balance of token = ${contract_addr} for use=${user_addr} `;
  let query = { balance: { address: user_addr } };
  return await queryContractDebug(
    terra,
    contract_addr,
    query,
    queryName,
    logPath
  );
}

async function get_reply_sub_msg(
  terra: LCDClient,
  logPath: fs.PathOrFileDescriptor,
  network: any
): Promise<any> {
  let queryName = "sub_msg ";
  let query = { reply_sub_msg_response: {} };
  return await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );
}

async function get_dca_config(
  terra: LCDClient,
  logPath: fs.PathOrFileDescriptor,
  network: any
): Promise<any> {
  let queryName = "config dca";
  let query = {
    config: {},
  };

  return await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );
}

async function get_dca_order_id(
  terra: LCDClient,
  id: string,
  logPath: fs.PathOrFileDescriptor,
  network: any
): Promise<any> {
  let queryName = `dca_orders with id = ${id} `;
  let query = { dca_orders: { id: id } };
  return await queryContractDebug(
    terra,
    network.DcaAddress,
    query,
    queryName,
    logPath
  );
}
