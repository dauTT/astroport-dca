import { strictEqual } from "assert"
import 'dotenv/config'
import {
    newClient,
    writeArtifact,
    readArtifact,
    queryContract,
    deployContract,
    executeContract,
    uploadContract, instantiateContract,
} from './helpers.js'
import { join } from 'path'



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
    Wallet
} from '@terra-money/terra.js';


const ARTIFACTS_PATH = 'astroport_artifacts'
const TOKEN_INITIAL_AMOUNT = String(1000_000_000_000);

const TOKENS_SYMBOLS = ["AAA", "BBB", "CCC", "DDD", "EEE", "FFF", "GGG", "HHH", "III"] 

interface balance {
    address: string 
    amount: string
}

interface logo {
    url: string 
}
interface marketing{
    project: string,
    description: string,
    marketing: string,
    logo: logo
}
interface tokenInfo {
    name: string,
    symbol: string,
    decimals: number,
    initial_balances: balance[]
    marketing: marketing

}

type SymbolToAddress = {
    [key: string]: string;
  };


function getTokenInfos(wallet: Wallet): tokenInfo[]  {
    let tokens: tokenInfo[] = []
    let balances: balance[] = []

    // provide to each test account an initial amount of token
    // equivalent to TOKEN_INITIAL_AMOUNT
    Object.values(LOCAL_TERRA_TEST_ACCOUNTS).forEach((addr)=>{
        balances.push({
            address: addr,
            amount: TOKEN_INITIAL_AMOUNT
        })
    })


    for (var t of TOKENS_SYMBOLS){
        tokens.push(
            {   name: `token ${t}`,
                symbol: t,
                decimals: 6,
                initial_balances: balances,
                marketing: {
                    project: `project ${t}`,
                    description: `description ${t}`,
                    marketing: wallet.key.accAddress,
                    logo: {
                        url: `https://website.${t}`
                    }
                }
            }

        )
    } 

    return tokens

}
 
const LOCAL_TERRA_TEST_ACCOUNTS = {
    test0: "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8",    
    test1:  "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
    test2:	"terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp",
    test3:	"terra1757tkx08n0cqrw7p86ny9lnxsqeth0wgp0em95",	
    test4:	"terra199vw7724lzkwz6lf2hsx04lrxfkz09tg8dlp6r",	
    test5:	"terra18wlvftxzj6zt0xugy2lr9nxzu402690ltaf4ss",	
    test6:	"terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9",	
    test7:	"terra17tv2hvwpg0ukqgd2y5ct2w54fyan7z0zxrm2f9",	
    test8:	"terra1lkccuqgj6sjwjn8gsa9xlklqv4pmrqg9dx2fxc",	
    test9:	"terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f",	
    test10:	"terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n"
}

async function main() {
    const { terra, wallet } = newClient()
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)
    const network = readArtifact(terra.config.chainID)
    console.log(`Token codeId: ${network.tokenCodeID}`)
  
    console.log('network:', network)

    let tokenInfos = getTokenInfos(wallet)
    if (!network.tokenAddresses){
        const tokenAddresses: SymbolToAddress = {}
        let tokenSymbols:String[] = []
        for (var tokenInfo of  tokenInfos){  
            console.log(`*************** process token : ${tokenInfo.symbol}$********************`)
            if (tokenAddresses !== {}) {
                 tokenSymbols = Object.keys(tokenAddresses)
            }

            if (!tokenSymbols.includes(tokenInfo.symbol)){
                 let label = `token label ${tokenInfo.symbol}`

                // console.log('tokenInfo', tokenInfo)
                 console.log("Instantiate  token contract")
                let resp = await instantiateContract(terra, wallet, wallet.key.accAddress, network.tokenCodeID, tokenInfo, label)
                
                // @ts-ignore
                let tokenAddress:string  = resp.shift().shift()

                tokenAddresses[tokenInfo.symbol] = tokenAddress;
                        
                console.log(tokenAddress)
                console.log(await queryContract(terra, tokenAddress, { token_info: {} }))
                console.log(await queryContract(terra, tokenAddress, { minter: {} }))

                let balance = await queryContract(terra, tokenAddress, { balance: { address: tokenInfo.initial_balances[0].address } })
                strictEqual(balance.balance, tokenInfo.initial_balances[0].amount)

                network.tokenAddresses =tokenAddresses
                writeArtifact(network, terra.config.chainID)
                console.log('FINISH')
            }
        }
    }


}


main().catch(console.log)
