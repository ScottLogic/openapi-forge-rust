const headerParametersExist = (params) => {
  return (
    Array.isArray(params) &&
    params.some((p) => p.in === "header" || p.name === "cookie")
  );
};

module.exports = headerParametersExist;
