const getSome = (is_cabi_str) => {
    let some = "";
    if (is_cabi_str == "true") {
        some = "Option::RSome"
    }
    else {
        some = "Some"
    }
    return some;
}

module.exports = getSome;
