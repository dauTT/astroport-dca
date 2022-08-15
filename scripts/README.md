## Scripts

### Build env local

```shell
npm install
npm start
```

### Deploy on `localterra`

Build contract:

```shell
npm run build-artifacts
```

Create `.env`:

```shell
WALLET="notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius"
LCD_CLIENT_URL=http://localhost:1317
CHAIN_ID=localterra

```

This file is located in `scripts` folder

run locaterra image: `dautt/astroport:v1.2.0`

```shell
// from the root of the project execute
cd scripts/tests/localterra/
local_terra_image.sh run

```

Make sure the local blockchain is running:

```shell
// from the root of the project execute
cd scripts/tests/localterra/
local_terra_image.sh enter

// type:
terrad status

// exit from the container iafter checking that the output is fine:
exit

```

After executing `terrad status` inside the running container you should see an output similar to this one. If this is the case then, the blockchain is running fine.

```
{"NodeInfo":{"protocol_version":{"p2p":"8","block":"11","app":"0"},"id":"62cd922a9d9349e790247dadd1e32947450502fb","listen_addr":"tcp://0.0.0.0:26656","network":"localterra","version":"v0.34.19-terra.2","channels":"40202122233038606100","moniker":"localterra","other":{"tx_index":"on","rpc_address":"tcp://0.0.0.0:26657"}},"SyncInfo":{"latest_block_hash":"7B4B7122E977F97A8026166CE299528CA937A0811282AB544F575549BEE34F2F","latest_app_hash":"671CD261D4DE866876CA7569E14508E25254AFD0A2059D98979702E3C2A4C59D","latest_block_height":"943","latest_block_time":"2022-08-15T21:02:12.9844774Z","earliest_block_hash":"73770BCC69947E89AC6099F7B3047E6DF47C042FBD944A387DAD5C8252CE68DD","earliest_app_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","earliest_block_height":"1","earliest_block_time":"2022-05-16T04:54:32.922225Z","catching_up":false},"ValidatorInfo":{"Address":"DA0317A8E3251C9AEAA38C34820568DCD030CF3F","PubKey":{"type":"tendermint/PubKeyEd25519","value":"/zGmkgCWRFsJLETAzlzYsbu7EHS5HWpaSyR22rlFM68="},"VotingPower":"1"}}

```

The image `dautt/astroport:v1.2.0` is configured with following addresses which are stored in the network.json file located
here `scripts/tests/localterra/network.json`

```json
{
  "tokenCodeID": 15,
  "tokenAddress": "terra17v6gz38e94ryfeewtlpafw0jd20afgsu8xu8px2dz7xaqkz4afrqshln05",
  "multisigAddress": "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
  "whitelistCodeID": 19,
  "whitelistAddress": "terra160uy8dt3wk6q5aswc266t6hak07xergpmakk9srsaedg7xwcw67s3yg939",
  "pairCodeID": 20,
  "pairStableCodeID": 21,
  "xastroTokenCodeID": 22,
  "stakingAddress": "terra1f5vdkslcghur6uqecar9rrg5qr6skmflprj6gzpaqxznjdah3syqqj5zu4",
  "xastroAddress": "terra1u9ayepulw285rfcj5vh462e0t6vwa6yxl7960l9pkae66w2gf7hqgwjcw9",
  "factoryAddress": "terra1qnmggqv3u03axw69cn578hukqhl4f2ze2k403ykcdlhj98978u7stv7fyj",
  "routerAddress": "terra15kwtyl2jtf8frwh3zu2jntqvem8u36y8aw6yy9z3ypgkfjx6ct2q73xas8",
  "makerAddress": "terra1ypsg336x2phzcwze4yu7ecv5ptu069rgxzane56njh502kuf9thskjdvqj",
  "tokenAddresses": {
    "AAA": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe",
    "BBB": "terra14haqsatfqxh3jgzn6u7ggnece4vhv0nt8a8ml4rg29mln9hdjfdq9xpv0p",
    "CCC": "terra1q0e70vhrv063eah90mu97sazhywmeegptx642t5px7yfcrf0rrsq2nesul",
    "DDD": "terra10v0hrjvwwlqwvk2yhwagm9m9h385spxa4s54f4aekhap0lxyekys4jh3s4",
    "EEE": "terra1wastjc07zuuy46mzzl3egz4uzy6fs59752grxqvz8zlsqccpv2wqnfu3yr",
    "FFF": "terra1uvt40rsp68wtas0y75w34qdn5h0g5eyefy5gmvzftdnupyv7q7vqqp9fge",
    "GGG": "terra1dkcsehtk7vq2ta9x4kdazlcpr4s58xfxt3dvuj98025rmleg4g2q8g6d3p",
    "HHH": "terra1ul4msjc3mmaxsscdgdtjds85rg50qrepvrczp0ldgma5mm9xv8yq8a7tex",
    "III": "terra19egn9e8v5493mtum626upcjyu5xj3mru57htge0ma66sk5gyqf4qu6r90m"
  },
  "poolAAA-BBB": "terra1d4sad30uj59lxg56ylwn7457v8z4k5m3323r9u360q85w8ga3kfsk2eyfs",
  "lpTokenAAA-BBB": "terra14ddekavkyjwfy0tdw9ng7rxxmw9mjpms2ppmav3nfh6akady3pzqlggg5t",
  "poolBBB-LUNA": "terra16frj05cr02nkwgdc45pawjvuqql8vs6dzz0x92y6ag0spt4sfd5s5dsl09",
  "lpTokenBBB-LUNA": "terra1qyl0j7a24amk8k8gcmvv07y2zjx7nkcwpk73js24euh64hkja6esv7qrld",
  "poolLUNA-CCC": "terra1qt0gkcrvcpv765k8ec4tl2svvg6hd3e3td8pvg2fsncrt3dzjefsa4n0jc",
  "lpTokenLUNA-CCC": "terra105espjya6qc7tazk8drsnvf2675q5wywafnwlhs5tpx9yza7hpmqqxpx35",
  "poolBBB-DDD": "terra1nl237syqtwr37nh3gf207fn3t8a5gfy4cp0puvt0s9fk4sm0qmzsuc4mc5",
  "lpTokenBBB-DDD": "terra1ggc4snw6dkj49n7j5znc9ae9lk92a8nlje9nkmrqryhwdzz0g50qssm3ql",
  "DcaCodeId": 31,
  "DcaAddress": "terra1zrkla3nzvenamj4vp95yxtw6cfxymxkgwqk79z997hqaeq8yg8wsdw47e2"
}
```

A sample script for deploying the dca contract on the locaterra blockchain is given here:
`scripts/deploy_astro_dca_ts`

If one wish to deploy the dca contract again, one needs to execute following two steps:

```
Remove the two line from the network.json file located here:
scripts/astroport_artifacts/network.json. (Remember to remove also the comma before `DcaCodeId`! )

,
"DcaCodeId": 31,
"DcaAddress": "terra..."

```

and then run

```shell
cd scripts
node --loader ts-node/esm deploy_astro_dca_ts

```

### e2e testing

Finally run all e2e testing by executing these cmd:

```
cd scripts
node --loader ts-node/esm tests/main.ts
```

For each test case, we log the output of the execution here: `scripts/logs`
