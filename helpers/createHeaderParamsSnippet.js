const Handlebars = require("handlebars");
const toParamName = require("./toParamName");
const getParametersByType = require("./getParametersByType");
const getSome = require("./getSome");

const pushToHeaderParam = (name, value) =>
  `headers.insert(${name}, reqwest::header::HeaderValue::from_str(&${value})?);`;

const createHeaderParamsSnippet = (sortedParams, is_cabi = false) => {
  let headerSnippet = `let mut headers = reqwest::header::HeaderMap::new();\n`;
  //Add cookie parameters
  let cookieParams = getParametersByType(sortedParams, "cookie");
  if (cookieParams.length !== 0) {
    let safeParamName = toParamName(cookieParams[0].name);
    cookieParams = cookieParams.slice(1);
    for (const cookieParam of cookieParams) {
      safeParamName = toParamName(cookieParam.name);
      headerSnippet += pushToHeaderParam(
        `reqwest::header::COOKIE`,
        `format!("{}={}", ${cookieParams[0].name},${safeParamName})`
      );
    }
    headerSnippet += "\n";
  }

  const headerParams = getParametersByType(sortedParams, "header");
  if (headerParams.length === 0) {
    return new Handlebars.SafeString(headerSnippet);
  }
  for (const headerParam of headerParams) {
    // only supports default serialization: style: simple & explode: false
    if (headerParam.content) {
      continue;
    }
    const safeParamName = toParamName(headerParam.name);
    switch (headerParam.schema.type) {
      case "array":
        headerSnippet += pushToHeaderParam(
          headerParam.name,
          `${safeParamName}.join(",")`
        );
        break;
      case "object": {
        let serialisedObject = "";
        for (const [propName] of Object.entries(
          headerParam.schema.properties
        )) {
          serialisedObject += `${propName},${safeParamName}.${propName}`;
        }
        headerSnippet += pushToHeaderParam(headerParam.name, serialisedObject);
        break;
      }
      default: {
        if (!headerParam.required) {
          headerSnippet +=
            `if let ` +
            getSome(is_cabi) +
            `(${safeParamName}) = ${safeParamName} { ` +
            pushToHeaderParam(`"${headerParam.name}"`, safeParamName) +
            `}`;
        } else {
          headerSnippet += pushToHeaderParam(
            `"${headerParam.name}"`,
            safeParamName
          );
        }
      }
    }
  }
  return new Handlebars.SafeString(headerSnippet);
};

module.exports = createHeaderParamsSnippet;
