
const urlPath = (url_str)  => {
    const url = new URL(url_str);
    return url.pathname;
}


module.exports = urlPath;