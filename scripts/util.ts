import * as fs from "fs";

export const ARTIFACTS_PATH = "astroport_artifacts";
export const LOGS_PATH = "logs";
export const TEST_LOGS_PATH = "../logs";

export const ASSET_CLASS = {
  token: "token",
  native_token: "native_token",
};

export const LOCAL_TERRA_TEST_ACCOUNTS: { [key: string]: any } = {
  // test0 = validator
  test0: {
    addr: "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8",
    mnemonic:
      "satisfy adjust timber high purchase tuition stool faith fine install that you unaware feed domain license impose boss human eager hat rent enjoy dawn",
  },
  test1: {
    addr: "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
    mnemonic:
      "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius",
  },
  test2: {
    addr: "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp",
    mnemonic:
      "quality vacuum heart guard buzz spike sight swarm shove special gym robust assume sudden deposit grid alcohol choice devote leader tilt noodle tide penalty",
  },
  test3: {
    addr: "terra1757tkx08n0cqrw7p86ny9lnxsqeth0wgp0em95",
    mnemonic:
      "symbol force gallery make bulk round subway violin worry mixture penalty kingdom boring survey tool fringe patrol sausage hard admit remember broken alien absorb",
  },
  test4: {
    addr: "terra199vw7724lzkwz6lf2hsx04lrxfkz09tg8dlp6r",
    mnemonic:
      "bounce success option birth apple portion aunt rural episode solution hockey pencil lend session cause hedgehog slender journey system canvas decorate razor catch empty",
  },
  test5: {
    addr: "terra18wlvftxzj6zt0xugy2lr9nxzu402690ltaf4ss",
    mnemonic:
      "second render cat sing soup reward cluster island bench diet lumber grocery repeat balcony perfect diesel stumble piano distance caught occur example ozone loyal",
  },
  test6: {
    addr: "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9",
    mnemonic:
      "spatial forest elevator battle also spoon fun skirt flight initial nasty transfer glory palm drama gossip remove fan joke shove label dune debate quick",
  },
  test7: {
    addr: "terra17tv2hvwpg0ukqgd2y5ct2w54fyan7z0zxrm2f9",
    mnemonic:
      "noble width taxi input there patrol clown public spell aunt wish punch moment will misery eight excess arena pen turtle minimum grain vague inmate",
  },
  test8: {
    addr: "terra1lkccuqgj6sjwjn8gsa9xlklqv4pmrqg9dx2fxc",
    mnemonic:
      "cream sport mango believe inhale text fish rely elegant below earth april wall rug ritual blossom cherry detail length blind digital proof identify ride",
  },
  test9: {
    addr: "terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f",
    mnemonic:
      "index light average senior silent limit usual local involve delay update rack cause inmate wall render magnet common feature laundry exact casual resource hundred",
  },
  test10: {
    addr: "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n",
    mnemonic:
      "prefer forget visit mistake mixture feel eyebrow autumn shop pair address airport diesel street pass vague innocent poem method awful require hurry unhappy shoulder",
  },
};

export function logToFile(
  filePath: fs.PathOrFileDescriptor,
  content: String,
  header: String = ""
) {
  let niceString =
    (header === "" ? `` : ` ${header} `) +
    ` 
  ${content}`;

  console.log(niceString);

  fs.writeFile(filePath, niceString, { flag: "a" }, function (err) {
    if (err) {
      console.log(err);
    }
  });
}

export function deleteFile(filePath: fs.PathLike) {
  try {
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath);
    }
  } catch (err) {
    console.error(err);
  }
}
