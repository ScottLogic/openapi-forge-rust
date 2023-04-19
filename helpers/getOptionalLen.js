const getOptionalLen = (len) => {
  if (typeof len !== "number") {
    len = 0;
  }
  return len;
};

module.exports = getOptionalLen;
