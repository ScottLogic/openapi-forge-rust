use abi_stable::std_types::{ RString, RVec };
use anyhow::{ bail, Context, Ok, Result };
use convert_case::Casing;
use cucumber::{ gherkin::Step, when };

use crate::{
    data::{ FFIObject, ParamWithType },
    ffi::{
        get_fn_signature,
        run_method_no_params,
        run_method_one_param,
        run_method_two_params,
        serialize_returned_variable,
        run_method_one_serialized_param,
    },
    mock::{ set_mock_with_json_response, set_mock_with_string_response },
    util::{ compile_generated_api, forge, hash_an_object, write_schema_to_file },
    ForgeWorld,
};

#[when(expr = "generating an API from the following specification")]
async fn api_specification_2(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    // This doesn't generate api client
    // schema
    if let Some(spec) = step.docstring() {
        let hash = hash_an_object(spec);
        w.library_name_modifier = Some(hash);
        write_schema_to_file(spec, w.library_name_modifier.context("library modifier")?).await?;
    } else {
        bail!("API spec not found");
    }
    // forge + compile + set
    forge(w.library_name_modifier.context("library modifier")?).await?;
    compile_generated_api(w.library_name_modifier.context("library modifier")?).await?;
    w.set_library()?;
    Ok(())
}

#[when(expr = "calling the method {word} without params")]
async fn call_method_without_params(w: &mut ForgeWorld, method_name: String) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    set_mock_with_string_response(&method_name).await?;
    run_method_no_params::<RString>(w, &method_name)?;
    Ok(())
}

#[when(expr = "calling the spied method {word} without params")]
async fn call_spied_method_without_params(w: &mut ForgeWorld, method_name: String) -> Result<()> {
    call_method_without_params(w, method_name).await?;
    Ok(())
}

#[when(expr = "calling the method {word} and the server responds with")]
async fn call_method_with_server_responds(
    w: &mut ForgeWorld,
    method_name: String,
    step: &Step
) -> Result<()> {
    // schema
    let raw_response_body = step.docstring().context("response body not found")?.trim();
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    set_mock_with_json_response(raw_response_body).await?;
    // run method
    let ffi_object = run_method_no_params::<FFIObject>(w, &method_name)?;
    let tuple = serialize_returned_variable::<FFIObject>(w, &method_name, ffi_object)?;
    w.last_object_response = Some(tuple);
    Ok(())
}

#[when(expr = "selecting the server at index {int}")]
async fn when_selecting_index(w: &mut ForgeWorld, idx: u8) -> Result<()> {
    w.set_reset_client(Some(idx))?;
    Ok(())
}

#[when(expr = "calling the method {word} with parameters {word}")]
async fn call_method_with_params(
    w: &mut ForgeWorld,
    method_name: String,
    params: String
) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let trimmed = &params[1..params.len() - 1];
    let list = trimmed.split(',').collect::<Vec<_>>();
    // add mock
    set_mock_with_string_response(&method_name).await?;
    // get fn signature
    let info = get_fn_signature(w, &method_name)?;
    // check if input params are correct length
    assert_eq!(list.len(), info.input_types.len());
    // collect params
    let params = list
        .iter()
        .zip(info.input_types)
        .filter_map(|(el, el_type)| ParamWithType::from(el, &el_type).ok())
        .collect::<Vec<_>>();
    // run method
    let ffi_object = match params.len() {
        1 =>
            match params[0].clone() {
                ParamWithType::Number(el) => run_method_one_param(w, &method_name, el)?,
                ParamWithType::OptionalNumber(el) => run_method_one_param(w, &method_name, el)?,
                ParamWithType::String(el) => run_method_one_param(w, &method_name, el)?,
                ParamWithType::OptionalString(el) => run_method_one_param(w, &method_name, el)?,
            }
        2 =>
            match (params[0].clone(), params[1].clone()) {
                (ParamWithType::String(el1), ParamWithType::String(el2)) => {
                    run_method_two_params(w, &method_name, el1, el2)?
                }
                (ParamWithType::String(el1), ParamWithType::OptionalString(el2)) => {
                    run_method_two_params(w, &method_name, el1, el2)?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::OptionalNumber(el2)) => {
                    run_method_two_params(w, &method_name, el1, el2)?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::String(el2)) => {
                    run_method_two_params(w, &method_name, el1, el2)?
                }
                (ParamWithType::OptionalString(el1), ParamWithType::OptionalString(el2)) => {
                    run_method_two_params(w, &method_name, el1, el2)?
                }
                _ => bail!("not covered all cases"),
            }
        _ => bail!("Too many arguments"),
    };
    let tuple = serialize_returned_variable::<FFIObject>(w, &method_name, ffi_object)?;
    w.last_object_response = Some(tuple);
    Ok(())
}

#[when(expr = "calling the method {word} with array {word}")]
async fn call_method_with_array(
    w: &mut ForgeWorld,
    method_name: String,
    array: String
) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let trimmed = &array[1..array.len() - 1];
    let list = trimmed
        .split(',')
        .map(|el| RString::from(el))
        .collect::<RVec<_>>();
    // add mock
    set_mock_with_string_response(&method_name).await?;
    // get fn signature
    let info = get_fn_signature(w, &method_name)?;
    // there should be one input type of Vec
    assert_eq!(info.input_types.len(), 1);
    assert!(info.input_types[0].contains("Vec"));
    let ffi_object = run_method_one_param(w, &method_name, list)?;
    let tuple = serialize_returned_variable::<FFIObject>(w, &method_name, ffi_object)?;
    w.last_object_response = Some(tuple);
    Ok(())
}

#[when(regex = r"calling the method (\S+) with object (.*)")]
async fn call_method_with_object(
    w: &mut ForgeWorld,
    method_name: String,
    json_str: String
) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    set_mock_with_string_response(&method_name).await?;
    // get fn signature
    let info = get_fn_signature(w, &method_name)?;
    // there should be one input type of InlineObject[0-9]*
    assert_eq!(info.input_types.len(), 1);
    assert!(info.input_types[0].contains("InlineObject"));
    let ffi_object = run_method_one_serialized_param(w, &method_name, RString::from(json_str))?;
    let tuple = serialize_returned_variable::<FFIObject>(w, &method_name, ffi_object)?;
    w.last_object_response = Some(tuple);
    Ok(())
}