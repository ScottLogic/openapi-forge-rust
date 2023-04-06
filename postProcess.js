const rename_files = require("./postProcessRenameFiles");

module.exports = (folder, _, options) => {
  rename_files(folder, options);
};
