import { strictEqual } from "assert";
import {
  newClient,
  writeArtifact,
  readArtifact,
  instantiateContract,
  queryContract,
  uploadContract,
  NativeAsset,
  TokenAsset,
} from "./helpers.js";

import { ARTIFACTS_PATH } from "./util.js";

const CONTRACT_LABEL = "Astroport dca module";
const DCA_MODULE__BINARY_PATH = `${ARTIFACTS_PATH}/astroport_dca_module.wasm`;

function init_msg(network: any): any {
  return {
    factory_addr:
      "terra1qnmggqv3u03axw69cn578hukqhl4f2ze2k403ykcdlhj98978u7stv7fyj",
    gas_info: {
      native_token: {
        denom: "uluna",
      },
    },
    max_hops: 3,
    max_spread: "0.5",
    owner: network.multisigAddress,
    per_hop_fee: "100000",
    router_addr: network.routerAddress,
    whitelisted_tokens: {
      source: [
        new TokenAsset(network.tokenAddresses.AAA).getInfo(),
        new TokenAsset(network.tokenAddresses.BBB).getInfo(),
        new TokenAsset(network.tokenAddresses.DDD).getInfo(),
        new NativeAsset("uluna").getInfo(),
      ],
      tip: [
        new TokenAsset(network.tokenAddresses.CCC).getInfo(),
        new TokenAsset(network.tokenAddresses.AAA).getInfo(),
        new NativeAsset("uluna").getInfo(),
      ],
    },
  };
}

// Main
async function main() {
  const { terra, wallet } = newClient();
  const network = readArtifact(terra.config.chainID);
  console.log(
    `chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`
  );
  console.log("network: ", network);

  // Upload DCA contract code
  if (!network.DcaCodeId) {
    console.log(`Upload contract: ${DCA_MODULE__BINARY_PATH}`);
    network.DcaCodeId = await uploadContract(
      terra,
      wallet,
      DCA_MODULE__BINARY_PATH
    );
    console.log(`DCA codeId: ${network.DcaCodeId}`);
    writeArtifact(network, terra.config.chainID);
  }

  // Instantiate DCA contract
  if (!network.DcaAddress) {
    let MSG = init_msg(network);
    console.log("Initiate contract msg:", JSON.stringify(MSG, null, 4));
    let resp = await instantiateContract(
      terra,
      wallet,
      wallet.key.accAddress,
      network.DcaCodeId,
      MSG,
      CONTRACT_LABEL
    );
    console.log("response: ", resp);

    // @ts-ignore
    network.DcaAddress = resp.shift().shift();
    console.log("DCA address:", network.DcaAddress);
    writeArtifact(network, terra.config.chainID);
  }

  let config = await queryContract(terra, network.DcaAddress, { config: {} });
  strictEqual(config.router_addr, network.routerAddress);

  console.log("config: ", JSON.stringify(config, null, 4));

  console.log("FINISH");
}

main().catch(console.log);
