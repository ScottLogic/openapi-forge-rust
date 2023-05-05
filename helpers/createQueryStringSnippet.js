const Handlebars = require("handlebars");
const toRustParamName = require("./toRustParamName");
const toParamName = require("./toParamName");
const getParametersByType = require("./getParametersByType");
const getSome = require("./getSome");

const isRequired = (typeDef) => {
  return typeof typeDef._required !== "undefined";
};

const pushToQueryParam = (name, value, is_cabi_str = false) => {
  const optionalToString = is_cabi_str === "true" ? ".to_string()" : "";
  const optionalInto = is_cabi_str === "true" ? ".into()" : "";
  return `query_params.push(("${name}"${optionalToString}.into(), ${toRustParamName(
    value
  )}${optionalToString}${optionalInto}));`;
};

const serialiseArrayParam = (param, is_cabi = false) => {
  const safeParamName = toParamName(param.name);
  const serialisedParam =
    `for el in ${safeParamName} {` +
    pushToQueryParam(safeParamName, `el`, is_cabi) +
    `}`;
  return serialisedParam;
};

const serialiseObjectParam = (param, is_required = false, is_cabi = false) => {
  const safeParamName = toParamName(param.name);
  let serialisedObject = "";
  for (const [propName, objProp] of Object.entries(param.schema.properties)) {
    let res = "";
    if (!isRequired(objProp)) {
      res =
        `if let ` +
        getSome(is_cabi) +
        `(${toRustParamName(propName)}) = &${safeParamName}.${toRustParamName(
          propName
        )} { ` +
        pushToQueryParam(propName, propName, is_cabi) +
        ` }`;
    } else {
      res = pushToQueryParam(propName, `${safeParamName}.${propName}`, is_cabi);
    }

    if (!is_required) {
      serialisedObject +=
        `if let ` +
        getSome(is_cabi) +
        `(${safeParamName}) = &${safeParamName} { ${res}  }`;
    } else {
      serialisedObject += res;
    }
  }

  return serialisedObject;
};

const serialisePrimitive = (param, is_required = false, is_cabi = false) => {
  const safeParamName = toParamName(param.name);
  const inner = pushToQueryParam(safeParamName, safeParamName, is_cabi);
  if (!is_required) {
    return (
      `if let ` +
      getSome(is_cabi) +
      `(${toRustParamName(safeParamName)}) = ${toRustParamName(
        safeParamName
      )} { ${inner}  }`
    );
  } else {
    return inner;
  }
};

const createQueryStringSnippet = (params, is_cabi = false) => {
  const queryParams = getParametersByType(params, "query");

  if (queryParams.length === 0) {
    return "";
  }

  let queryStringSnippet = `let mut query_params : Vec<(String, String)> = Vec::new();`;

  for (const queryParam of queryParams) {
    let serialisedQueryParam;
    switch (queryParam.schema.type) {
      case "array":
        serialisedQueryParam = serialiseArrayParam(queryParam, is_cabi);
        break;
      case "object":
        serialisedQueryParam = serialiseObjectParam(
          queryParam,
          queryParam.required,
          is_cabi
        );
        break;
      default:
        serialisedQueryParam = serialisePrimitive(
          queryParam,
          queryParam.required,
          is_cabi
        );
        break;
    }

    queryStringSnippet += serialisedQueryParam;
  }

  return new Handlebars.SafeString(queryStringSnippet);
};

module.exports = createQueryStringSnippet;
