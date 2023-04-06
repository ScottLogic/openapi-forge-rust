const Handlebars = require("handlebars");
const typeConvert = require("./typeConvert");

const safeTypeConvert = (prop, is_required = true) => {
  if (typeof is_required !== "boolean") {
    is_required = true;
  }
  return new Handlebars.SafeString(typeConvert(prop, is_required));
};

module.exports = safeTypeConvert;
