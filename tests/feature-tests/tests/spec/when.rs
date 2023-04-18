use abi_stable::std_types::{ROption, RString};
use anyhow::{bail, Context, Result};
use convert_case::Casing;
use cucumber::{gherkin::Step, when};

use crate::{
    data::FFIObject,
    ffi::{
        run_method_no_params_with_return, run_method_one_param, run_method_two_params,
        serialize_returned_variable,
    },
    util::{compile_generated_api, forge, hash_an_object, write_schema_to_file},
    ForgeWorld, mock::{set_mock_with_string_response, set_mock_with_json_response},
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
    run_method_no_params_with_return::<RString>(w, &method_name)?;
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
    step: &Step,
) -> Result<()> {
    // schema
    let raw_response_body = step.docstring().context("response body not found")?.trim();
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    set_mock_with_json_response(raw_response_body).await?;
    // run method
    let ffi_object = run_method_no_params_with_return::<FFIObject>(w, &method_name)?;
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
    params: String,
) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let trimmed = &params[1..params.len() - 1];
    let list = trimmed.split(',').collect::<Vec<_>>();
    // add mock
    set_mock_with_string_response(&method_name).await?;
    // run method
    match list.len() {
        1 => match list[0].parse::<i32>() {
            Ok(i) => run_method_one_param(w, &method_name, ROption::RSome(i))?,
            Err(_) => {
                run_method_one_param(w, &method_name, ROption::RSome(RString::from(list[0])))?
            }
        },
        2 => match (list[0].parse::<i32>(), list[1].parse::<i32>()) {
            (Ok(i), Ok(j)) => {
                run_method_two_params(w, &method_name, ROption::RSome(i), ROption::RSome(j))?
            }
            (Ok(i), Err(_)) => run_method_two_params(
                w,
                &method_name,
                ROption::RSome(i),
                ROption::RSome(RString::from(list[1])),
            )?,
            (Err(_), Ok(j)) => run_method_two_params(
                w,
                &method_name,
                ROption::RSome(RString::from(list[0])),
                ROption::RSome(j),
            )?,
            (Err(_), Err(_)) => run_method_two_params(
                w,
                &method_name,
                ROption::RSome(RString::from(list[0])),
                ROption::RSome(RString::from(list[1])),
            )?,
        },
        3 => {
            todo!()
        }
        _ => bail!("Too many arguments"),
    };
    Ok(())
}
