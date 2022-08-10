import 'dotenv/config'
import {
    newClient,
    writeArtifact,
    readArtifact,
    deployContract,
    executeContract,
    queryContract,
    toEncodedBinary,
    NativeAsset,
    TokenAsset
} from './helpers.js'
import { join } from 'path'

const ARTIFACTS_PATH = 'astroport_artifacts'


async function main() {
    const { terra, wallet } = newClient()
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)
    const network = readArtifact(terra.config.chainID)
    console.log('network:', network)

    if (network.tokenAddress == "") {
        throw new Error("token address is not set, create ASTRO token first")
    }

    let pools =  [
        {
            identifier: "AAA-BBB",
            assetInfos: [
                new TokenAsset(network.tokenAddresses.AAA).getInfo(),
                new TokenAsset(network.tokenAddresses.BBB).getInfo(),
            ],
            pairType: { xyk: {} }
        },
        {
            identifier: "BBB-LUNA",
            assetInfos: [
                new TokenAsset(network.tokenAddresses.BBB).getInfo(),
                new NativeAsset("uluna").getInfo(),
            ],
            pairType: { stable: {} },
            initParams: toEncodedBinary({ amp: 100 })
        },
        {
            identifier: "LUNA-CCC",
            assetInfos: [
                new NativeAsset("uluna").getInfo(),
                new TokenAsset(network.tokenAddresses.CCC).getInfo(),
            ],
            pairType: { xyk: {} },
            initGenerator: {
                generatorAllocPoint: 1000000
            }
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
            }
        },
    ]

    for (let i = 0; i < pools.length; i++) {
        let pool = pools[i]
        let pool_pair_key = "pool" + pool.identifier
        let pool_lp_token_key = "lpToken" + pool.identifier

        // Create pool
        if (!network[pool_pair_key]) {
            console.log(`Creating pool ${pool.identifier}...`)
            let res = await executeContract(terra, wallet, network.factoryAddress, {
                create_pair: {
                    pair_type: pool.pairType,
                    asset_infos: pool.assetInfos,
                    init_params: pool.initParams
                }
            })

            console.log("response create_pair: ", res)
            console.log("********** response create_pair string **************: ", JSON.stringify(res, null, 4))


           network[pool_pair_key] = res.logs[0].eventsByType.wasm.pair_contract_addr[0]
           network[pool_lp_token_key] = res.logs[0].eventsByType.wasm.liquidity_token_addr[0]

           /*
            let pool_info = await queryContract(terra, network[pool_pair_key], {
                pair: {}
            })

            network[pool_lp_token_key] = pool_info.liquidity_token
            */

            console.log("Pair successfully created!")
            console.log(`Pool Address: ${network[pool_pair_key]}`)
            console.log(`lpToken Address: ${network[pool_lp_token_key]}`)

            writeArtifact(network, terra.config.chainID)
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

    console.log('FINISH')
}

main().catch(console.log)
