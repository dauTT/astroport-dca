import * as fs from "fs";

export function logToFile(filePath: fs.PathOrFileDescriptor, content: String, header = "") {
  fs.writeFile(
    filePath,
    (header === "" ? `` : ` ${header} `) +
      ` 
    ${content}
    `,
    { flag: "a" },
    function (err) {
      if (err) {
        console.log(err);
      }
    }
  );
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
