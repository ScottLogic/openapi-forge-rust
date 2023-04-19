const camelToSnakeCase = (str) =>
  str.replace(/[A-Z0-9]/g, (letter) => `_${letter.toLowerCase()}`);

const toRustParamName = (name) => {
  name = name.replace(/[^a-z0-9]/gi, "");
  name = camelToSnakeCase(name);
  // type is reserved keyword
  if (name == "type") {
    name = "r#" + name;
  }
  return name;
};

module.exports = toRustParamName;
