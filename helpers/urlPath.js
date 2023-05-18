const urlPath = (url_str) => {
  try {
    const url = new URL(url_str);
    return url.pathname;
  } catch (err) {
    // if cannot parse url, assume it's already path
    return url_str;
  }
};

module.exports = urlPath;
