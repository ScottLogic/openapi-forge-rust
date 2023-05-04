const toSafeName = require("./toClassName");

const fromFormat = (propFormat, is_required) => {
  switch (propFormat) {
    case "int32":
      return is_required ? "i32" : "Option<i32>";
    case "int64":
      return is_required ? "i64" : "Option<i64>";
    case "float":
      return is_required ? "f32" : "Option<f32>";
    case "double":
      return is_required ? "f64" : "Option<f64>";
    case "date":
      const naiveDate = "chrono::naive::NaiveDate";
      return is_required ? naiveDate : `Option<${naiveDate}>`;
    case "date-time":
      const dateTime = "chrono::DateTime<chrono::Utc>";
      return is_required ? dateTime : `Option<${dateTime}>`;
    case "byte":
    case "binary":
    case "string":
      return is_required ? "String" : "Option<String>";
    default:
      return is_required ? "()" : "Option<()>";
  }
};

const fromType = (propType, additionalProperties, items, is_required) => {
  switch (propType) {
    case "integer":
      return is_required ? "i64" : "Option<i64>";
    case "number":
      return is_required ? "f64" : "Option<f64>";
    case "boolean":
      return is_required ? "bool" : "Option<bool>";
    case "string":
      return is_required ? "String" : "Option<String>";
    case "array":
      return `Vec<${typeConvert(items, true)}>`;
    // inline object definition
    case "object":
      if (additionalProperties) {
        return `HashMap<String,${typeConvert(
          additionalProperties,
          is_required
        )}>`;
      } else {
        return "Object";
      }
    default:
      return "";
  }
};

const typeConvert = (prop, is_required = true) => {
  if (prop === null) return "()";

  if (prop === undefined) return "()";

  // resolve references
  if (prop.$ref) {
    const objectType = toSafeName(prop.$ref.split("/").pop());
    return is_required ? objectType : `Option<${objectType}>`;
  }

  const type = prop.format
    ? fromFormat(prop.format, is_required)
    : fromType(prop.type, prop.additionalProperties, prop.items, is_required);

  return type === "" ? "()" : type;
};

module.exports = typeConvert;
