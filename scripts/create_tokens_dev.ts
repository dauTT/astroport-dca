import { strictEqual } from "assert";
import "dotenv/config";
import {
  newClient,
  writeArtifact,
  readArtifact,
  queryContract,
  deployContract,
  executeContract,
  uploadContract,
  instantiateContract,
} from "./helpers.js";
import { join } from "path";

import {
  Int,
  // Coin,
  // Coins,
  // isTxError,
  LCDClient,
  /* LocalTerra,
    MnemonicKey,
    Msg,
    MsgExecuteContract,
    MsgInstantiateContract,
    MsgMigrateContract,
    MsgStoreCode,
    MsgUpdateContractAdmin, Tx,
    */
  Wallet,
} from "@terra-money/terra.js";

import { LOCAL_TERRA_TEST_ACCOUNTS, ARTIFACTS_PATH } from "./util.js";

// const ARTIFACTS_PATH = "astroport_artifacts";
const TOKEN_INITIAL_AMOUNT = String(1000_000_000_000);

const TOKENS_SYMBOLS = [
  "AAA",
  "BBB",
  "CCC",
  "DDD",
  "EEE",
  "FFF",
  "GGG",
  "HHH",
  "III",
];

interface balance {
  address: string;
  amount: string;
}

interface logo {
  url: string;
}
interface marketing {
  project: string;
  description: string;
  marketing: string;
  logo: logo;
}
interface tokenInfo {
  name: string;
  symbol: string;
  decimals: number;
  initial_balances: balance[];
  marketing: marketing;
}

type SymbolToAddress = {
  [key: string]: string;
};

function getTokenInfos(wallet: Wallet): tokenInfo[] {
  let tokens: tokenInfo[] = [];
  let balances: balance[] = [];

  // provide to each test account an initial amount of token
  // equivalent to TOKEN_INITIAL_AMOUNT
  Object.values(LOCAL_TERRA_TEST_ACCOUNTS).forEach((value) => {
    balances.push({
      address: value.addr,
      amount: TOKEN_INITIAL_AMOUNT,
    });
  });

  for (var t of TOKENS_SYMBOLS) {
    tokens.push({
      name: `token ${t}`,
      symbol: t,
      decimals: 6,
      initial_balances: balances,
      marketing: {
        project: `project ${t}`,
        description: `description ${t}`,
        marketing: wallet.key.accAddress,
        logo: {
          url: `https://website.${t}`,
        },
      },
    });
  }

  return tokens;
}

async function main() {
  const { terra, wallet } = newClient();
  console.log(
    `chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`
  );
  const network = readArtifact(terra.config.chainID);
  console.log(`Token codeId: ${network.tokenCodeID}`);

  console.log("network:", network);

  let tokenInfos = getTokenInfos(wallet);
  if (!network.tokenAddresses) {
    const tokenAddresses: SymbolToAddress = {};
    let tokenSymbols: String[] = [];
    for (var tokenInfo of tokenInfos) {
      console.log(
        `*************** process token : ${tokenInfo.symbol}$********************`
      );
      if (tokenAddresses !== {}) {
        tokenSymbols = Object.keys(tokenAddresses);
      }

      if (!tokenSymbols.includes(tokenInfo.symbol)) {
        let label = `token label ${tokenInfo.symbol}`;

        // console.log('tokenInfo', tokenInfo)
        console.log("Instantiate  token contract");
        let resp = await instantiateContract(
          terra,
          wallet,
          wallet.key.accAddress,
          network.tokenCodeID,
          tokenInfo,
          label
        );

        // @ts-ignore
        let tokenAddress: string = resp.shift().shift();

        tokenAddresses[tokenInfo.symbol] = tokenAddress;

        console.log(tokenAddress);
        console.log(
          await queryContract(terra, tokenAddress, { token_info: {} })
        );
        console.log(await queryContract(terra, tokenAddress, { minter: {} }));

        let balance = await queryContract(terra, tokenAddress, {
          balance: { address: tokenInfo.initial_balances[0].address },
        });
        strictEqual(balance.balance, tokenInfo.initial_balances[0].amount);

        network.tokenAddresses = tokenAddresses;
        writeArtifact(network, terra.config.chainID);
        console.log("FINISH");
      }
    }
  }
}

main().catch(console.log);
