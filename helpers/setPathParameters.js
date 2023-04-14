const Handlebars = require("handlebars");
const toRustParamName = require("./toRustParamName");
const getParametersByType = require("./getParametersByType");
const getSome = require("./getSome");

const setPathParameters = (path, sortedParams, is_cabi = false) => {
  const pathParams = getParametersByType(sortedParams, "path");
  if (pathParams.length === 0) {
    return `"` + path + `"`;
  }

  let res = new Handlebars.SafeString(
    path.replace(/{(.*?)}/g, (match, captureGroup) => {
      var pathParam = pathParams.find((p) => p.name === captureGroup);

      if (pathParam === undefined) {
        throw `helper setPathParameters: cannot find PATH parameter named '${captureGroup}' in available path parameters: ${pathParams.map(
          (p) => `'${p.name}'`
        )}`;
      }

      const safeParamName = toRustParamName(captureGroup);
      const URL_SAFE_COMMA = "%2C";
      switch (pathParam.schema.type) {
        case "array":
          return `" + ${safeParamName}.iter().join("${URL_SAFE_COMMA}") + "`;
        case "object": {
          let serialisedObject = "";
          for (const [propName] of Object.entries(
            pathParam.schema.properties
          )) {
            serialisedObject += `${propName}${URL_SAFE_COMMA}" + ${safeParamName}.${propName}.into() + "${URL_SAFE_COMMA}`;
          }
          return serialisedObject.slice(0, -3);
        }
        default: {
          if (pathParam.required) {
            return `", &${safeParamName}.to_string(), "`;
          } else {
            return (
              `", &{ if let ` +
              getSome(is_cabi) +
              `(${safeParamName}) = ${safeParamName} { ${safeParamName}.to_string() } else { "".into() } }, "`
            );
          }
        }
      }
    })
  );
  res = `&["${res}"].join("")`;
  return res;
};

module.exports = setPathParameters;
