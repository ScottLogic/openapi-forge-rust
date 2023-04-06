const Handlebars = require("handlebars");
const toParamName = require("./toParamName");
const getParametersByType = require("./getParametersByType");

const isStringType = (typeDef) =>
  typeDef.type === "string" &&
  (typeDef.format === undefined || typeDef.format === "string");

const isStringArrayParam = (param) =>
  param.schema.type === "array" &&
  param.schema.items &&
  isStringType(param.schema.items);

const serialiseArrayParam = (param, is_required = false) => {
  const safeParamName = toParamName(param.name);
  const serialisedParam = `for el in ${safeParamName} {
    query_params.push(("${param.name}".to_owned(), el));
  }`;
  return serialisedParam;
};

const serialiseObjectParam = (param, is_required = false) => {
  const safeParamName = toParamName(param.name);
  let serialisedObject = "";
  for (const [propName, objProp] of Object.entries(param.schema.properties)) {
    let optional_amp = serialisedObject.length == 0 ? "" : `"&"+`;
    // open first parantheses for the conditional to make it easy to read
    // open second parantheses for making sure to combine the string in a conditional block
    let nullCheck;
    if (param.required) {
      nullCheck = "(";
    } else {
      nullCheck = `${safeParamName}.${propName} == null ? "" : (`;
    }
    let serialisedParam = `(` + nullCheck + optional_amp + `"${propName}="`;
    let suffix = isStringType(objProp)
      ? `+ java.net.URLEncoder.encode(${safeParamName}.${propName}, StandardCharsets.UTF_8)`
      : `+ ${safeParamName}.${propName}`;
    // close both parantheses
    serialisedObject += serialisedParam + suffix + "))+";
  }

  return `${prefixSerialisedQueryParam(serialisedObject.slice(0, -1), safeParamName)}`;
};

const serialisePrimitive = (param, is_required = false) => {
  const safeParamName = toParamName(param.name);
  const inner = `query_params.push(("${param.name}".into(), ${safeParamName}.into()));`;
  if (!is_required) {
    return `if let Some(${safeParamName}) = ${safeParamName} { ${inner}  }`
  }
  else {
    return inner;
  }
};

const prefixSerialisedQueryParam = (serialisedQueryParam, safeParamName) => {
  return `if (${safeParamName} != null) { ${indent}queryString.append((queryString.length() == 0 ? "?" : "&").concat(${serialisedQueryParam})); }`;
};

const createQueryStringSnippet = (params) => {
  const queryParams = getParametersByType(params, "query");

  if (queryParams.length === 0) {
    return "";
  }

  let queryStringSnippet = `let mut query_params : Vec<(String, String)> = Vec::new();`;

  for (const queryParam of queryParams) {
    let serialisedQueryParam;
    switch (queryParam.schema.type) {
      case "array":
        serialisedQueryParam = serialiseArrayParam(queryParam, queryParam.schema._required);
        break;
      case "object":
        serialisedQueryParam = serialiseObjectParam(queryParam, queryParam.schema._required);
        break;
      default:
        serialisedQueryParam = serialisePrimitive(queryParam, queryParam.schema._required);
        break;
    }

    queryStringSnippet +=  serialisedQueryParam;
  }

  return new Handlebars.SafeString(queryStringSnippet);
};

module.exports = createQueryStringSnippet;
