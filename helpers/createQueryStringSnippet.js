const Handlebars = require("handlebars");
const toParamName = require("./toParamName");
const getParametersByType = require("./getParametersByType");
const getSome = require("./getSome")

const isRequired = (typeDef) => {
  return typeof typeDef._required !== 'undefined';
}

const pushToQueryParam = (name, value) => 
  `query_params.push(("${name}".to_string().into(), ${value}.to_string().into()));`

const serialiseArrayParam = (param, is_required = false) => {
  const safeParamName = toParamName(param.name);
  const serialisedParam = `for el in ${safeParamName} {`+
    pushToQueryParam(safeParamName, `el`) + 
  `}`;
  return serialisedParam;
};

const serialiseObjectParam = (param, is_required = false, is_cabi = false) => {
  const safeParamName = toParamName(param.name);
  let serialisedObject = "";
  for (const [propName, objProp] of Object.entries(param.schema.properties)) {
    let res = "";
    if (!isRequired(objProp)){
      res = `if let ` + getSome(is_cabi) + `(${propName}) = ${safeParamName}.${propName} { ` + pushToQueryParam(propName, propName) + ` }`; 
    }
    else {
      res = pushToQueryParam(propName, `${safeParamName}.${propName}`);
    }

    if (!is_required) {
      return `if let ` + getSome(is_cabi) + `(${safeParamName}) = ${safeParamName} { ${res}  }`
    }
    else {
      return res;
    }
  }

  return `${serialisedObject.slice(0, -1)}`;
};

const serialisePrimitive = (param, is_required = false, is_cabi = false) => {
  const safeParamName = toParamName(param.name);
  const inner = pushToQueryParam(safeParamName, safeParamName);
  if (!is_required) {
    return `if let ` + getSome(is_cabi) + `(${safeParamName}) = ${safeParamName} { ${inner}  }`
  }
  else {
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
        serialisedQueryParam = serialiseArrayParam(queryParam, queryParam.schema._required, is_cabi);
        break;
      case "object":
        serialisedQueryParam = serialiseObjectParam(queryParam, queryParam.schema._required, is_cabi);
        break;
      default:
        serialisedQueryParam = serialisePrimitive(queryParam, queryParam.schema._required, is_cabi);
        break;
    }

    queryStringSnippet +=  serialisedQueryParam;
  }

  return new Handlebars.SafeString(queryStringSnippet);
};

module.exports = createQueryStringSnippet;
