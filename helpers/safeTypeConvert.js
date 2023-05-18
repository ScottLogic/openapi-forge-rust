const Handlebars = require("handlebars");
const typeConvert = require("./typeConvert");

const safeTypeConvert = (prop, is_required = true, is_cabi_str = "false") => {
  if (typeof is_required !== "boolean") {
    is_required = true;
  }
  const is_cabi = is_cabi_str === "true";
  return new Handlebars.SafeString(typeConvert(prop, is_required, is_cabi));
};

module.exports = safeTypeConvert;
