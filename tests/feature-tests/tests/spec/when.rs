use abi_stable::std_types::{ RString, RVec };
use anyhow::{ bail, Context, Ok, Result };
use convert_case::Casing;
use cucumber::{ gherkin::Step, when };
use serde_json::Value;

use crate::{
    data::{ FFIObject, FFISafeTuple },
    ffi::{ call::FFICaller, dispatch::FFIDispatcher },
    mock::ForgeMockServer,
    util::{ compile_generated_api, forge, hash_an_object, write_schema_to_file },
    ForgeWorld,
};

#[when(expr = "generating an API from the following specification")]
async fn api_specification_generation(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    // This doesn't generate api client
    // schema
    let spec = step.docstring().context("API spec not found")?;
    let spec_hash = hash_an_object(spec);
    w.library_name_modifier = Some(spec_hash);
    write_schema_to_file(spec, w.library_name_modifier.context("library modifier")?).await?;
    // forge + compile + set
    forge(w.library_name_modifier.context("library modifier")?).await?;
    compile_generated_api(w.library_name_modifier.context("library modifier")?).await?;
    w.set_library()?;
    Ok(())
}

#[when(expr = "calling the method {word} without params")]
async fn call_method_without_params(w: &mut ForgeWorld, method_name: String) -> Result<()> {
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    let method_name = method_name.to_case(convert_case::Case::Snake);
    ForgeMockServer::set_mock_with_string_response(&method_name).await?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    let fn_signature = FFICaller::get_fn_signature(w, &api_client_name, &method_name)?;
    let params = FFIDispatcher::get_fn_params(&vec![], &fn_signature.input_types);
    match fn_signature.return_type.as_str() {
        "String" => {
            let response = FFIDispatcher::get_response::<RString>(
                w,
                &api_client_name,
                &method_name,
                params
            )?;
            let inner = FFICaller::returned_value_to_inner(
                w,
                &api_client_name,
                &method_name,
                response
            )?;
            w.last_string_response = Some(*inner);
        }
        _complex => {
            let ffi_object = FFIDispatcher::get_response::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                params
            )?;
            let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                ffi_object
            )?;
            w.last_object_response = Some(tuple);
        }
    }
    w.api_client_name = Some(api_client_name);
    w.last_fn_call_sign = Some(fn_signature);
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
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    // schema
    let raw_response_body = step.docstring().context("response body not found")?.trim();
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    ForgeMockServer::set_mock_with_json_response(raw_response_body).await?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    // fn
    let info = FFICaller::get_fn_signature(w, &api_client_name, &method_name)?;
    // run method
    match info.return_type.as_str() {
        "String" => {
            let response = FFIDispatcher::get_response::<RString>(
                w,
                &api_client_name,
                &method_name,
                vec![]
            )?;
            let inner = FFICaller::returned_value_to_inner(
                w,
                &api_client_name,
                &method_name,
                response
            )?;
            w.last_string_response = Some(*inner);
        }
        _complex => {
            let ffi_object = FFIDispatcher::get_response::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                vec![]
            )?;
            let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                ffi_object
            )?;
            w.last_object_response = Some(tuple);
        }
    }
    w.api_client_name = Some(api_client_name);
    w.last_fn_call_sign = Some(info);
    Ok(())
}

#[when(expr = "selecting the server at index {int}")]
async fn when_selecting_index(w: &mut ForgeWorld, idx: u8) -> Result<()> {
    w.set_reset_client(Some(idx), None)?;
    Ok(())
}

#[when(expr = "calling the method {word} with parameters {word}")]
async fn call_method_with_params(
    w: &mut ForgeWorld,
    method_name: String,
    params: String
) -> Result<()> {
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let trimmed = &params[1..params.len() - 1];
    let list = trimmed.split(',').collect::<Vec<_>>();
    let api_client_name = w.api_client_name.take().context("No client name")?;
    // add mock
    ForgeMockServer::set_mock_with_string_response(&method_name).await?;
    // get fn signature
    let info = FFICaller::get_fn_signature(w, &api_client_name, &method_name)?;
    let return_type = &info.return_type;
    // collect params
    let params = FFIDispatcher::get_fn_params(&list, &info.input_types);
    // run method
    match return_type.as_str() {
        "String" => {
            let response = FFIDispatcher::get_response::<RString>(
                w,
                &api_client_name,
                &method_name,
                params
            )?;
            let inner = FFICaller::returned_value_to_inner(
                w,
                &api_client_name,
                &method_name,
                response
            )?;
            w.last_string_response = Some(*inner);
        }
        _complex => {
            let ffi_object = FFIDispatcher::get_response::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                params
            )?;
            let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                ffi_object
            )?;
            w.last_object_response = Some(tuple);
        }
    }
    w.api_client_name = Some(api_client_name);
    Ok(())
}

#[when(expr = "calling the method {word} with array {word}")]
async fn call_method_with_array(
    w: &mut ForgeWorld,
    method_name: String,
    array: String
) -> Result<()> {
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let trimmed = &array[1..array.len() - 1];
    let list = trimmed
        .split(',')
        .map(|el| RString::from(el))
        .collect::<RVec<_>>();
    // add mock
    ForgeMockServer::set_mock_with_string_response(&method_name).await?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    // get fn signature
    let info = FFICaller::get_fn_signature(w, &api_client_name, &method_name)?;
    // there should be one input type of Vec
    assert_eq!(info.input_types.len(), 1);
    assert!(info.input_types[0].contains("Vec"));
    // put info into world
    w.last_fn_call_sign = Some(info);
    let ffi_object = FFICaller::run_method_one_param(w, &api_client_name, &method_name, list)?;
    let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
        w,
        &api_client_name,
        &method_name,
        ffi_object
    )?;
    w.last_object_response = Some(tuple);
    w.api_client_name = Some(api_client_name);
    Ok(())
}

#[when(regex = r"calling the method (\S+) with object (.*)")]
async fn call_method_with_object(
    w: &mut ForgeWorld,
    method_name: String,
    json_str: String
) -> Result<()> {
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    ForgeMockServer::set_mock_with_string_response(&method_name).await?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    // get fn signature
    let info = FFICaller::get_fn_signature(w, &api_client_name, &method_name)?;
    // there should be one input type of InlineObject[0-9]* or ObjectResponse
    assert_eq!(info.input_types.len(), 1);
    assert!(info.input_types[0].contains("Object"));
    let ffi_object = FFICaller::run_method_one_serialized_param(
        w,
        &api_client_name,
        &method_name,
        RString::from(json_str)
    )?;
    let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
        w,
        &api_client_name,
        &method_name,
        ffi_object
    )?;
    w.last_object_response = Some(tuple);
    w.last_fn_call_sign = Some(info);
    w.api_client_name = Some(api_client_name);
    Ok(())
}

#[when(expr = "extracting the object at index {int}")]
async fn choose_index_of_array(w: &mut ForgeWorld, idx: usize) -> Result<()> {
    // alter fn return sign
    if let Some(fn_sign) = &mut w.last_fn_call_sign {
        assert!(fn_sign.return_type.contains("Vec"));
        // Remove Vec< and >
        let new_return = RString::from(fn_sign.return_type.slice(4..fn_sign.return_type.len() - 1));
        fn_sign.return_type = new_return;
    } else {
        bail!("no fn sign");
    }
    // alter object
    let object = w.last_object_response.take();
    if let Some(last) = object {
        let serialized = &last.1;
        let mut value = serde_json::from_str::<Value>(serialized.as_str())?;
        let data = value.get_mut("data").context("data container")?;
        let array = data.as_array_mut().context("cannot take array")?;
        let extracted = array.remove(idx);
        *data = extracted;
        let extracted_serialized = RString::from(value.to_string());
        w.last_object_response = Some(FFISafeTuple(last.0, extracted_serialized));
    } else {
        bail!("no last response");
    }

    Ok(())
}

#[when(expr = "calling the method {word} and the server responds with headers")]
async fn call_method_with_server_responds_headers(
    w: &mut ForgeWorld,
    method_name: String,
    step: &Step
) -> Result<()> {
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let headers = step.docstring().context("response body not found")?.trim();
    let header_value = serde_json::from_str::<Value>(&headers)?;
    let header_object = header_value
        .as_object()
        .context("object")?
        .iter()
        .filter_map(|(k, v)| Some((&k[..], v.as_str()?)))
        .collect::<Vec<_>>();
    // add mock
    ForgeMockServer::set_mock_with_header(header_object[0]).await?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    // fn
    let info = FFICaller::get_fn_signature(w, &api_client_name, &method_name)?;
    // run method
    match info.return_type.as_str() {
        "String" => {
            let response = FFIDispatcher::get_response::<RString>(
                w,
                &api_client_name,
                &method_name,
                vec![]
            )?;
            let inner = FFICaller::returned_value_to_inner(
                w,
                &api_client_name,
                &method_name,
                response
            )?;
            w.last_string_response = Some(*inner);
        }
        _complex => {
            let ffi_object = FFIDispatcher::get_response::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                vec![]
            )?;
            let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
                w,
                &api_client_name,
                &method_name,
                ffi_object
            )?;
            w.last_object_response = Some(tuple);
        }
    }
    w.api_client_name = Some(api_client_name);
    w.last_fn_call_sign = Some(info);
    Ok(())
}

#[when(expr = "calling the method {word} and the server provides an empty response")]
async fn call_method_with_server_responds_empty(
    w: &mut ForgeWorld,
    method_name: String
) -> Result<()> {
    // make sure api_client exists
    if w.api_client.is_none() {
        w.set_reset_client(None, None)?;
    }
    let method_name = method_name.to_case(convert_case::Case::Snake);
    ForgeMockServer::set_mock_empty().await?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    let ffi_object = FFIDispatcher::get_response::<FFIObject>(
        w,
        &api_client_name,
        &method_name,
        vec![]
    )?;
    let tuple = FFICaller::serialize_returned_variable::<FFIObject>(
        w,
        &api_client_name,
        &method_name,
        ffi_object
    )?;
    w.last_object_response = Some(tuple);
    w.api_client_name = Some(api_client_name);
    Ok(())
}