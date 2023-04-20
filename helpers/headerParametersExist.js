const headerParametersExist = (params) => {
  return (
    Array.isArray(params) &&
    params.some((p) => p.in === "header" || p.in === "cookie")
  );
};

module.exports = headerParametersExist;
