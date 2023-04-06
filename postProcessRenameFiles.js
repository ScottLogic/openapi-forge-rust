const shell = require("shelljs");

const camelToSnakeCase = str => str.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`);

// Lowercase and snake case ApiClient rs files

module.exports = (folder, _) => {
  shell.cd(folder + "/src/api_client");
  shell.ls('*.rs').forEach(function (file) {
    shell.mv(file, camelToSnakeCase(file).toLowerCase());
  });
};
