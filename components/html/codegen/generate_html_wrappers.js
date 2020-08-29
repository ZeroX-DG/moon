const { parse } = require("webidl2");
const fs = require("fs");
const glob = require("glob");

function main() {
  glob("../idl/HTML*Element.idl", {}, function(er, files) {
    if (er) {
      return console.log("Error while find idl files");
    }

    const generateTasks = [];
    for (file of files) {
      generateTasks.push(
        new Promise((resolve, reject) => {
          fs.readFile(file, "utf8", function(err, data) {
            if (err) {
              return reject(console.log("Error while readingn idl file"));
            }
            const elementInterface = parse(data)[0];
            const wrapper = gerenateWrapper(elementInterface);
            const fileName = elementInterface.name.toLowerCase();

            fs.writeFile(`../src/elements/${fileName}.rs`, wrapper, err => {
              if (err) {
                reject(
                  console.log(
                    `Error while writing ${elementInterface.name}.rs: ${err}`
                  )
                );
                return;
              }
              console.log(
                `Generated ${elementInterface.name} -> ${fileName}.rs`
              );
              resolve(fileName);
            });
          });
        })
      );
    }

    Promise.all(generateTasks).then(fileNames => {
      const modFileContent = fileNames.reduce((acc, curr) => {
        acc += `mod ${curr};\n`;
        return acc;
      }, "");
      fs.writeFile(`../src/elements/mod.rs`, modFileContent, err => {
        if (err) {
          console.log("Error while writing mod.rs");
          return;
        }
        console.log("Generated mod file. All good to go!");
      });
    });
  });
}

function gerenateWrapper(elementInterface) {
  let output = `pub struct ${elementInterface.name} {\n`;
  for (member of elementInterface.members) {
    if (member.type == "attribute") {
      const attribute_type = member.idlType.idlType;
      output += `  ${member.name}: ${mapTypeToRustType(attribute_type)},\n`;
    }
  }
  output += "}";
  return output;
}

function mapTypeToRustType(type) {
  if (type == "DOMString") {
    return "String";
  }
  return type;
}

main();
