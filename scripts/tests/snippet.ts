// https://github.com/nodejs/node/blob/v17.0.0/lib/assert.js
import { strictEqual } from "assert";

import "dotenv/config";
import { queryContractDebug, queryBankDebug, TokenAsset } from "../helpers.js";
import * as fs from "fs";

import {
  initTestClient,
  getDcaConfig,
  getTokenBalance,
  getDcaOrderId,
} from "./common.js";

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
async function snippet() {
  const { terra, wallet, network, logPath } = initTestClient(
    "snippet",
    "test1"
  );

  /*
  await queryBankDebug(
    terra,
    network.DcaAddress,
    " luna balance for user oder",
    logPath
  );

  let userAddr = network.DcaAddress; // LOCAL_TERRA_TEST_ACCOUNTS["test1"].addr;
  await queryBankDebug(terra, userAddr, " luna balance for user", logPath);

  await getTokenBalance(
    terra,
    network.tokenAddresses.AAA,
    userAddr,
    logPath
  );

  await getDcaConfig(terra, network, logPath);
  await getDcaOrderId(terra, "2", network, logPath);

  await getReplySubMsg(terra, network, logPath);

  await queryPool(
    terra,
    new TokenAsset(network.tokenAddresses.AAA).getInfo(),
    new TokenAsset(network.tokenAddresses.BBB).getInfo(),
    network,
    logPath
  );

  */
}

snippet().catch(console.log);

async function queryPool(
  terra: LCDClient,
  asset1_info: any,
  asset2_info: any,
  network: any,
  logPath: fs.PathOrFileDescriptor
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

async function getReplySubMsg(
  terra: LCDClient,
  network: any,
  logPath: fs.PathOrFileDescriptor
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
