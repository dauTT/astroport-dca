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
  queryContractDebug,
  toEncodedBinary,
  performTransactions,
  NativeAsset,
  TokenAsset,
} from "../helpers.js";
import { join } from "path";

import { logToFile, deleteFile, TEST_LOGS_PATH } from "../util.js";
import * as fs from "fs";

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

async function main() {
  const { terra, wallet } = newClient();
  console.log(
    `chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`
  );
  const network = readArtifact("..", terra.config.chainID);
  console.log("network:", network);

  let logPath = `${TEST_LOGS_PATH}/queries.log`;
  deleteFile(logPath);

  let queryName: String;
  let query: any;

  queryName = "pair: AAA-BBB";
  query = {
    pair: {
      asset_infos: [
        new TokenAsset(network.tokenAddresses.AAA).getInfo(),
        new TokenAsset(network.tokenAddresses.BBB).getInfo(),
      ],
    },
  };

  await queryContractDebug(
    terra,
    network.factoryAddress,
    query,
    queryName,
    logPath
  );

  queryName = "pool: AAA-BBB";
  query = {
    pool: {},
  };

  await queryContractDebug(
    terra,
    network["poolAAA-BBB"],
    query,
    queryName,
    logPath
  );
}

main().catch(console.log);
