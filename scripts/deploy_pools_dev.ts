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
  NativeAsset,
  TokenAsset,
  Asset,
  performTransactions,
} from "./helpers.js";

import { strictEqual } from "assert";
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

import { LOGS_PATH, logToFile, deleteFile } from "./util.js";

import { LOCAL_TERRA_TEST_ACCOUNTS, ASSET_CLASS } from "./util.js";

const LOG_PATH = `${LOGS_PATH}/decploy_pools_dev.log`;

function init_asset(assetInfo: any, amount: string): Asset {
  if (assetInfo.hasOwnProperty(ASSET_CLASS.native_token)) {
    type ObjectKey = keyof typeof assetInfo;
    const native_token = ASSET_CLASS.native_token as ObjectKey;
    return new NativeAsset(assetInfo[native_token].denom, amount);
  } else {
    type ObjectKey = keyof typeof assetInfo;
    const token = ASSET_CLASS.token as ObjectKey;
    return new TokenAsset(assetInfo[token].contract_addr, amount);
  }
}

async function provide_liquidity(
  terra: LCDClient,
  wallet: Wallet,
  poolAddr: string,
  asset1: Asset,
  asset2: Asset
) {
  // msg allowance source: contract = network.tokenAddresses.AAA
  let msgIncreaseAllowance1: any = undefined;
  let msgIncreaseAllowance2: any = undefined;
  let coins: Coins.Input = [];
  // let coins2: Coins.Input = [];

  /*
  if (asset1.info === "native") {
    coins1 = [new Coin("uusd", asset1.amount)];
  }
  */

  // 4 combination
  // native, native
  // native, token
  // token, native
  // token, token

  let funds;
  if (
    asset1.getInfo().hasOwnProperty(ASSET_CLASS.native_token) &&
    asset2.getInfo().hasOwnProperty(ASSET_CLASS.native_token)
  ) {
    coins = [
      (asset1 as NativeAsset).toCoin(),
      (asset2 as NativeAsset).toCoin(),
    ];
  } else if (
    asset1.getInfo().hasOwnProperty(ASSET_CLASS.native_token) &&
    asset2.getInfo().hasOwnProperty(ASSET_CLASS.token)
  ) {
    coins = [(asset1 as NativeAsset).toCoin()];
    msgIncreaseAllowance2 = {
      increase_allowance: {
        spender: poolAddr,
        amount: asset2.getAsset().amount,
      },
    };
  } else if (
    asset1.getInfo().hasOwnProperty(ASSET_CLASS.token) &&
    asset2.getInfo().hasOwnProperty(ASSET_CLASS.native_token)
  ) {
    msgIncreaseAllowance1 = {
      increase_allowance: {
        spender: poolAddr,
        amount: asset1.getAsset().amount,
      },
    };

    coins = [(asset2 as NativeAsset).toCoin()];
  } else if (
    asset1.getInfo().hasOwnProperty(ASSET_CLASS.token) &&
    asset2.getInfo().hasOwnProperty(ASSET_CLASS.token)
  ) {
    msgIncreaseAllowance1 = {
      increase_allowance: {
        spender: poolAddr,
        amount: asset1.getAsset().amount,
      },
    };
    msgIncreaseAllowance2 = {
      increase_allowance: {
        spender: poolAddr,
        amount: asset2.getAsset().amount,
      },
    };
  } else {
    throw new Error(
      `Invalid asset combination: asset1.info=${asset1.getInfo()}, asset2.info=${asset2.getInfo()} `
    );
  }

  let msgProvideLiquidity = {
    provide_liquidity: {
      assets: [asset1.getAsset(), asset2.getAsset()],
    },
  };

  let msgs: MsgExecuteContract[] = [];

  if (msgIncreaseAllowance1 !== undefined) {
    msgs.push(
      new MsgExecuteContract(
        wallet.key.accAddress,
        asset1.getDenom(), // it will return the address of the token
        msgIncreaseAllowance1,
        []
      )
    );
  }

  if (msgIncreaseAllowance2 !== undefined) {
    msgs.push(
      new MsgExecuteContract(
        wallet.key.accAddress,
        asset2.getDenom(), // it will return the address of the token
        msgIncreaseAllowance2,
        []
      )
    );
  }

  msgs.push(
    new MsgExecuteContract(
      wallet.key.accAddress,
      poolAddr, // it will return the address of the token
      msgProvideLiquidity,
      coins
    )
  );

  logToFile(
    LOG_PATH,
    JSON.stringify(msgs, null, 4),
    "*************** msgs: *************"
  );

  let res = await performTransactions(terra, wallet, msgs);

  logToFile(
    LOG_PATH,
    JSON.stringify(res, null, 4),
    "**************** result tx:  ************"
  );
}

async function main() {
  const { terra, wallet } = newClient();
  console.log(
    `chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`
  );
  const network = readArtifact(terra.config.chainID);
  console.log("network:", network);
  deleteFile(LOG_PATH);

  if (network.tokenAddress == "") {
    throw new Error("token address is not set, create ASTRO token first");
  }

  let pools = [
    {
      identifier: "AAA-BBB",
      assetInfos: [
        new TokenAsset(network.tokenAddresses.AAA).getInfo(),
        new TokenAsset(network.tokenAddresses.BBB).getInfo(),
      ],
      pairType: { xyk: {} },
    },
    {
      identifier: "BBB-LUNA",
      assetInfos: [
        new TokenAsset(network.tokenAddresses.BBB).getInfo(),
        new NativeAsset("uluna").getInfo(),
      ],
      pairType: { stable: {} },
      initParams: toEncodedBinary({ amp: 100 }),
    },
    {
      identifier: "LUNA-CCC",
      assetInfos: [
        new NativeAsset("uluna").getInfo(),
        new TokenAsset(network.tokenAddresses.CCC).getInfo(),
      ],
      pairType: { xyk: {} },
      initGenerator: {
        generatorAllocPoint: 1000000,
      },
    },
    {
      identifier: "BBB-DDD",
      assetInfos: [
        new TokenAsset(network.tokenAddresses.BBB).getInfo(),
        new TokenAsset(network.tokenAddresses.DDD).getInfo(),
      ],
      pairType: { xyk: {} },
      initOracle: true,
      initGenerator: {
        generatorAllocPoint: 1000000,
      },
    },
  ];

  for (let i = 0; i < pools.length; i++) {
    let pool = pools[i];
    let pool_pair_key = "pool" + pool.identifier;
    let pool_lp_token_key = "lpToken" + pool.identifier;

    // Create pool
    if (!network[pool_pair_key]) {
      logToFile(
        LOG_PATH,
        JSON.stringify(`Creating pool ${pool.identifier}...`, null, 4),
        "*************** response create_pair: *************"
      );

      let res = await executeContract(terra, wallet, network.factoryAddress, {
        create_pair: {
          pair_type: pool.pairType,
          asset_infos: pool.assetInfos,
          init_params: pool.initParams,
        },
      });

      console.log("response create_pair: ", res);

      logToFile(
        LOG_PATH,
        JSON.stringify(res, null, 4),
        "*************** response create_pair: *************"
      );

      network[pool_pair_key] =
        res.logs[0].eventsByType.wasm.pair_contract_addr[0];
      network[pool_lp_token_key] =
        res.logs[0].eventsByType.wasm.liquidity_token_addr[0];

      /*
            let pool_info = await queryContract(terra, network[pool_pair_key], {
                pair: {}
            })

            network[pool_lp_token_key] = pool_info.liquidity_token
            */

      logToFile(
        LOG_PATH,
        `Pool Address: ${network[pool_pair_key]}
                 lpToken Address: ${network[pool_lp_token_key]}
               `,
        "*************** Pair successfully created!: *************"
      );

      writeArtifact(network, terra.config.chainID);

      logToFile(
        LOG_PATH,
        "",
        `****Provide liquididity to pool: ${pool.identifier} ********`
      );

      // test10 account will provide liquidity for all pools.
      // current balance test10:
      // 1000'000'000'000'000 uluna,
      // 1000'000'000'000 for each token: "AAA", "BBB", "CCC", "DDD", "EEE", "FFF", "GGG", "HHH","III",
      let asset1 = init_asset(pool.assetInfos[0], "100000000000");
      let asset2 = init_asset(pool.assetInfos[1], "100000000000");

      await provide_liquidity(
        terra,
        newTestClient("test10").wallet,
        network[pool_pair_key],
        asset1,
        asset2
      );
    }

    /*
        // Deploy oracle
        let pool_oracle_key = "oracle" + pool.identifier
        if (pool.initOracle && network[pool_pair_key] && !network[pool_oracle_key]) {
            console.log(`Deploying oracle for ${pool.identifier}...`)

            let resp = await deployContract(terra, wallet, network.multisigAddress, join(ARTIFACTS_PATH, 'astroport_oracle.wasm'), {
                factory_contract: network.factoryAddress,
                asset_infos: pool.assetInfos
            })
            network[pool_oracle_key] = resp.shift();

            console.log(`Address of ${pool.identifier} oracle contract: ${network[pool_oracle_key]}`)
            writeArtifact(network, terra.config.chainID)
        }

        */

    /*
        // Initialize generator
        if (network[pool_pair_key] && network[pool_lp_token_key] && pool.initGenerator) {
            let pool_generator_proxy_key = "generatorProxy" + pool.identifier
            network[pool_generator_proxy_key] = undefined

        
            if (pool.initGenerator.generatorProxy) {
                // Deploy proxy contract
                console.log(`Deploying generator proxy for ${pool.identifier}...`)
                let resp = await deployContract(terra, wallet, network.multisigAddress, join(ARTIFACTS_PATH, pool.initGenerator.generatorProxy.artifactName), {
                    generator_contract_addr: network.generatorAddress,
                    pair_addr: network[pool_pair_key],
                    lp_token_addr: network[pool_lp_token_key],
                    reward_contract_addr: pool.initGenerator.generatorProxy.rewardContractAddr,
                    reward_token_addr: pool.initGenerator.generatorProxy.rewardTokenAddr
                })
                network[pool_generator_proxy_key] = resp.shift();
                console.log(`Address of ${pool.identifier} generator proxy contract ${network[pool_generator_proxy_key]}`)

                // Set generator proxy as allowed
                let config = await queryContract(terra, network.generatorAddress, {
                    config: {}
                })
                let new_allowed_proxies: Array<String> = config.allowed_reward_proxies
                new_allowed_proxies.push(network[pool_generator_proxy_key] as String)
                console.log(`Set the proxy as allowed in generator... Allowed proxies with new one: ${new_allowed_proxies}`)
                await executeContract(terra, wallet, network.generatorAddress, {
                    set_allowed_reward_proxies: {
                        proxies: new_allowed_proxies
                    }
                })

            }

            

            // Add pool to generator
            console.log(`Adding ${pool.identifier} to generator...`)
            await executeContract(terra, wallet, network.generatorAddress, {
                add: {
                    alloc_point: String(pool.initGenerator.generatorAllocPoint),
                    reward_proxy: network[pool_generator_proxy_key],
                    lp_token: network[pool_lp_token_key]
                }
            })
        }
        */
  }

  // AAA-BBB
  // BBB-LUNA
  // LUNA-CCC
  // BBB-DDD

  // After providing liquidity. test10 will have following balance:
  // 999'980'000'000'000 uluna
  //     900'000'000'000 AAA
  //     700'000'000'000 BBB
  //     900'000'000'000 CCC
  //     900'000'000'000 DDD
  //    1000'000'000'000 for each token EEE, FFF, GGG, HHH

  sanity_checks(terra, wallet, network);

  console.log("FINISH");
}

main().catch(console.log);

async function sanity_checks(terra: LCDClient, wallet: Wallet, network: any) {
  // AAA-BBB
  // BBB-LUNA
  // LUNA-CCC
  // BBB-DDD

  // After providing liquidity. test10 will have following balance:
  // 999'980'000'000'000 uluna
  //     900'000'000'000 AAA
  //     700'000'000'000 BBB
  //     900'000'000'000 CCC
  //     900'000'000'000 DDD
  //    1000'000'000'000 for each token EEE, FFF, GGG, HHH

  try {
    let queryName = `luna balance of test10 account (${LOCAL_TERRA_TEST_ACCOUNTS.test10.addr})`;
    let res = await queryBankDebug(
      terra,
      LOCAL_TERRA_TEST_ACCOUNTS.test10.addr,
      queryName,
      LOG_PATH
    );

    let flag =
      BigInt("999800000000000") - BigInt(res[0]._coins.uluna.amount) <
      BigInt("20000000000"); // small difference due to gas consumption
    strictEqual(flag, true, queryName);

    queryName = `balance AAA token of of test10 account (${LOCAL_TERRA_TEST_ACCOUNTS.test10.addr})`;
    let query = {
      balance: {
        address: LOCAL_TERRA_TEST_ACCOUNTS.test10.addr,
      },
    };
    res = await queryContractDebug(
      terra,
      network.tokenAddresses.AAA,
      query,
      queryName,
      LOG_PATH
    );
    strictEqual(res.balance, "900000000000", queryName.toString());

    queryName = `balance BBB token of of test10 account (${LOCAL_TERRA_TEST_ACCOUNTS.test10.addr})`;
    res = await queryContractDebug(
      terra,
      network.tokenAddresses.BBB,
      query,
      queryName,
      LOG_PATH
    );
    strictEqual(res.balance, "700000000000", queryName.toString());

    queryName = `balance CCC token of of test10 account (${LOCAL_TERRA_TEST_ACCOUNTS.test10.addr})`;
    res = await queryContractDebug(
      terra,
      network.tokenAddresses.CCC,
      query,
      queryName,
      LOG_PATH
    );
    strictEqual(res.balance, "900000000000", queryName.toString());

    queryName = `balance DDD token of of test10 account (${LOCAL_TERRA_TEST_ACCOUNTS.test10.addr})`;
    res = await queryContractDebug(
      terra,
      network.tokenAddresses.DDD,
      query,
      queryName,
      LOG_PATH
    );
    strictEqual(res.balance, "900000000000", queryName.toString());
  } catch (err) {
    console.error(err);
    logToFile(
      LOG_PATH,
      String(err) + ": " + JSON.stringify(err, null, 4),
      "*********** something bad happened: error **************"
    );
  }
}
