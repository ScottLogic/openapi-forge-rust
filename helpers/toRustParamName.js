const camelToSnakeCase = str => str.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`);

const toRustParamName = (name) => {
  name = name.replace(/[^a-z0-9]/gi, "");
  name = camelToSnakeCase(name);
  // type is reserved keyword
  if (name == "type"){
    name = "r#" + name;
  }
  return name.charAt(0).toLowerCase() + name.substr(1);
};

module.exports = toRustParamName;
