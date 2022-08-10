
import 'dotenv/config'
import {
    newClient,
    writeArtifact,
    readArtifact,
    deployContract,
    executeContract,
    queryContract,
    toEncodedBinary,
    performTransactions,
    NativeAsset,
    TokenAsset
} from './helpers.js'
import { join } from 'path'

import { logToFile, deleteFile } from "./util.js";
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
    MsgUpdateContractAdmin, Tx,
    Wallet
} from '@terra-money/terra.js';

const ARTIFACTS_PATH = 'astroport_artifacts'


async function queryContractDebug(terra: LCDClient, contractAddress: string, query: object, queryName: String, logPath: fs.PathOrFileDescriptor = "") : Promise<any> {
     let res = await queryContract(terra, contractAddress, query)
  
    const header = `************** ${queryName} **************`;
    const niceOutput = JSON.stringify(res, null, 4);
    if (logPath !== "") {
      logToFile(logPath, niceOutput, header);
    }

    console.log(header);
    console.log(niceOutput);

    return res

}

async function getBlockTimeInSeconds(terra: LCDClient): Promise<number> {
    let blockInfo = await terra.utils.lcd.tendermint.blockInfo()
    let dateString = blockInfo.block.header.time 
    let seconds = Date.parse(dateString)
    let date = new Date(dateString)
    // getTime return the date in milliseconds since January 1, 1970, 00:00:00 UTC.
    return  Math.round(date.getTime() / 1000)
}

/*
async function executeContractDebug(terra: LCDClient, wallet: Wallet, contractAddress: string, msg: object, coins: Coins.Input = [], queryName: String, logPath: fs.PathOrFileDescriptor = "")  {


   await executeContract(terra, wallet, contractAddress, msg, coins)
 
   const header = `************** ${queryName} **************`;
   const niceOutput = JSON.stringify(res, null, 4);
   if (logPath !== "") {
     logToFile(logPath, niceOutput, header);
   }

   console.log(header);
   console.log(niceOutput);

   return res

}
*/


async function main() {

    const { terra, wallet } = newClient()
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)
    const network = readArtifact(terra.config.chainID)
    console.log('network:', network)
    
    const logPath = `astroport_artifacts/${terra.config.chainID}.log`;
    deleteFile(logPath);
    logToFile(logPath, JSON.stringify(network, null, 4), "**************** network **********************");
    
    let queryName: String 
    let query: any 

    /*
    queryName="pair: AAA-BBB"
    query = {
        pair: {
            asset_infos:[
                new TokenAsset(network.tokenAddresses.AAA).getInfo(),
                new TokenAsset(network.tokenAddresses.BBB).getInfo()
            ]
        }
    }
    await queryContractDebug(terra, network.factoryAddress, query, queryName, logPath)

    queryName="pair: BBB-LUNA"
    query = {
        pair: {
            asset_infos: [
                new TokenAsset(network.tokenAddresses.BBB).getInfo(),
                new NativeAsset("uluna").getInfo()
            ]
        }
    }
    await queryContractDebug(terra, network.factoryAddress, query, queryName, logPath)

    queryName="pair: LUNA-CCC"
    query = {
        pair: {
            asset_infos: [
                new NativeAsset("uluna").getInfo(),
                new TokenAsset(network.tokenAddresses.CCC).getInfo(),
            ]
        }
    }
    await queryContractDebug(terra, network.factoryAddress, query, queryName, logPath)

*/

    console.log ("***************** Testing DCA contract ************************")

    queryName = "config"
    query= { config: {} }
    await queryContractDebug(terra, network.DcaAddress, query, queryName, logPath)


    let blocktime = await getBlockTimeInSeconds(terra)

    

    //BEFORE: query balance
    queryName = `BEFORE: balance AAA token of the owner (sender): ${wallet.key.accAddress} `
    query= {
        balance: {
            address: wallet.key.accAddress
        }
    }
    await queryContractDebug(terra, network.tokenAddresses.AAA, query, queryName, logPath)


    //BEFORE: query allowance
    queryName = `BEFORE: allowance for DCA contract ${wallet.key.accAddress} to user AAA token of the owner ${wallet.key.accAddress}`
    query= {
        allowance: {
            owner: wallet.key.accAddress,
            spender: network.DcaAddress
        }
    }
    await queryContractDebug(terra, network.tokenAddresses.AAA, query, queryName, logPath)

    queryName = `BEFORE: balance AAA token of the DCA contract ${network.DcaAddress} `
    query= {
        balance: {
            address: network.DcaAddress
        }
    }
    await queryContractDebug(terra, network.tokenAddresses.AAA, query, queryName, logPath)


    // msg allowance source: contract = network.tokenAddresses.AAA
    let msgIncreaseAllowanceSource = {
        "increase_allowance": {
            spender: network.DcaAddress,
            amount: "10000000"
            
        }
    }

    // msg transferFrom source: contract = network.tokenAddresses.AAA
    let msgTrasnferFromSource = {
        transfer_from: {
            owner: wallet.key.accAddress,
            recipient: network.DcaAddress,
            amount: "10000000",
        }
    }
    // msg allowace tip: contract = network.tokenAddresses.AAA
     let msgIncreaseAllowanceTip = {
        "increase_allowance": {
            spender: network.DcaAddress,
            amount: "10000000"
            
        }
    }
    // msg transferFrom tip: contract = network.tokenAddresses.AAA
    let msgTrasnferFromTip = {
        transfer_from: {
            owner: wallet.key.accAddress,
            recipient: network.DcaAddress,
            amount: "10000000",
        }
    }
    // msg create order
    let gas = new  NativeAsset ('uluna', "1000000")
    let msgCreateDcaOrder = {
        create_dca_order: {
            dca_amount: new TokenAsset(network.tokenAddresses.AAA, "1000000").getInfo(),
            source: new TokenAsset(network.tokenAddresses.AAA, "10000000").getAsset() ,
            gas: gas.getAsset(),
            interval: 10,
            start_at: blocktime + 10,
            target_info: new TokenAsset(network.tokenAddresses.FFF).getInfo(),
            tip: new TokenAsset(network.tokenAddresses.AAA, "10000000").getAsset()
        }
    }

    let msgs =[
        // new MsgExecuteContract(wallet.key.accAddress, network.tokenAddresses.AAA, msgIncreaseAllowanceSource, []), // Allow the dca contract to spend money (Source)
        new MsgExecuteContract(network.DcaAddress, network.tokenAddresses.AAA, msgTrasnferFromSource, []),  // Transfer the money (Source) from the owner to the dca contract
        // new MsgExecuteContract(wallet.key.accAddress, network.tokenAddresses.AAA, msgIncreaseAllowanceTip, []), // Allow the dca contract to spend money (Tip)
        // new MsgExecuteContract(wallet.key.accAddress, network.tokenAddresses.AAA, msgTrasnferFromTip, []), // Transfer the money (Tip) from the owner to the dca contract
        // new MsgExecuteContract(wallet.key.accAddress, network.DcaAddress, msgCreateDcaOrder, [gas.toCoin()]) // Transfer the money (Gas) to the dca contract and create dca order
    ] 

    let header = "*************** msgs: create_dca_order *************"
    let msg_string = JSON.stringify(msgs, null, 4)
    console.log(header)
    console.log(msg_string)
    logToFile(logPath, JSON.stringify(msg_string, null, 4), header);


    let res =  await performTransactions(terra, wallet,  msgs)
    console.log("**************** result tx:  ************")
    console.log(res)
    logToFile(logPath, JSON.stringify(msg_string, null, 4), header);


    // AFTER: query balance
    queryName = `AFTER: balance AAA token of the owner (sender): ${wallet.key.accAddress} `
    query= {
        balance: {
            address: wallet.key.accAddress
        }
    }
    await queryContractDebug(terra, network.tokenAddresses.AAA, query, queryName, logPath)

    queryName = `AFTER: balance AAA token of the DCA contract ${network.DcaAddress} `
    query= {
        balance: {
            address: network.DcaAddress
        }
    }
    await queryContractDebug(terra, network.tokenAddresses.AAA, query, queryName, logPath)


    //AFTER: query allowance
    queryName = `AFTER: allowance for dca contract ${wallet.key.accAddress} to user AAA token of the owner ${wallet.key.accAddress}`
    query= {
        allowance: {
            owner: wallet.key.accAddress,
            spender: network.DcaAddress
        }
    }
    await queryContractDebug(terra, network.tokenAddresses.AAA, query, queryName, logPath)


    /*
    queryName = "dca_orders with id =1 "
    query= { dca_orders: {id: "1"} }
    await queryContractDebug(terra, network.DcaAddress, query, queryName, logPath)
    */




    /*
    queryName = "dca_orders with id =1 "
    query= { dca_orders: {id: "1"} }
    await queryContractDebug(terra, network.DcaAddress, query, queryName, logPath)
    */

    
    // executeContract(terra, wallet.key.accAddress, network.DcaAddress, msg: object, coins?: Coins.Input) {


  //  executeContract(terra, wallet.key.accAddress, network.DcaAddress, msg: object, coins?: Coins.Input) 






}

main().catch(console.log)