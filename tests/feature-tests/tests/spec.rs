use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

use abi_stable::std_types::{ROption, RString};
use anyhow::{bail, Context, Result};
use convert_case::Casing;
use cucumber::{gherkin::Step, given, then, when};
use serde_json::{json, Value};
use url::Url;
use wiremock::http::Method;
use wiremock::{matchers::any, Mock, ResponseTemplate};

use crate::data::FFIObject;
use crate::ffi::{
    get_type_information, get_type_name, run_method_no_params_with_return, run_method_one_param,
    run_method_two_params, serialize_returned_variable,
};
use crate::SERVER;
use crate::{util, ForgeWorld};

#[given(expr = "an API with the following specification")]
async fn api_specification(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    // schema
    if let Some(spec) = step.docstring() {
        let hash = util::hash_an_object(spec);
        w.library_name_modifier = Some(hash);
        util::write_schema_to_file(spec, w.library_name_modifier.context("library modifier")?)
            .await?;
    } else {
        bail!("API spec not found");
    }
    // forge + compile + set
    util::forge(w.library_name_modifier.context("library modifier")?).await?;
    util::compile_generated_api(w.library_name_modifier.context("library modifier")?).await?;
    w.set_library()?;
    w.set_reset_client(None)?;
    Ok(())
}
#[when(expr = "generating an API from the following specification")]
async fn api_specification_2(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    // This doesn't generate api client
    // schema
    if let Some(spec) = step.docstring() {
        let hash = util::hash_an_object(spec);
        w.library_name_modifier = Some(hash);
        util::write_schema_to_file(spec, w.library_name_modifier.context("library modifier")?)
            .await?;
    } else {
        bail!("API spec not found");
    }
    // forge + compile + set
    util::forge(w.library_name_modifier.context("library modifier")?).await?;
    util::compile_generated_api(w.library_name_modifier.context("library modifier")?).await?;
    w.set_library()?;
    Ok(())
}

#[when(expr = "calling the method {word} without params")]
async fn call_method_without_params(w: &mut ForgeWorld, method_name: String) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    if let Some(server) = SERVER.get() {
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_string({
                let a = format!("{}", method_name);
                // dbg!(&a);
                a
            }))
            .expect(1)
            .mount(server)
            .await;
    }
    // run method
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
    let response_body = step.docstring().context("response body not found")?.trim();
    let method_name = method_name.to_case(convert_case::Case::Snake);
    // add mock
    if let Some(server) = SERVER.get() {
        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(response_body, "application/json"),
            )
            .expect(1)
            .mount(server)
            .await;
    }
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
    if let Some(server) = SERVER.get() {
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_json("{}"))
            .expect(1)
            .mount(server)
            .await;
    }
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
        _ => panic!("Too many arguments"),
    };
    Ok(())
}

#[then(expr = "the requested URL should be {word}")]
async fn requested_url(_w: &mut ForgeWorld, url: String) -> Result<()> {
    if let Some(server) = SERVER.get() {
        if let Some(req) = server.received_requests().await {
            assert!(req.len() > 0);
            let last_req = &req[req.len() - 1];
            let expected_url = Url::parse(&url)?;
            let actual_url = &last_req.url;
            // only check the path + query
            let expected_path = Path::new(expected_url.path());
            let actual_path = Path::new(actual_url.path());
            assert_eq!(expected_path, actual_path);
            let expected_query = expected_url.query_pairs().collect::<HashSet<_>>();
            let actual_query = actual_url.query_pairs().collect::<HashSet<_>>();
            assert_eq!(expected_query, actual_query);
        }
    }

    // remove mocks
    if let Some(server) = SERVER.get() {
        server.reset().await;
    }
    Ok(())
}

#[then(expr = "the request method should be of type {word}")]
async fn requested_type_should_be(_w: &mut ForgeWorld, request_type: String) -> Result<()> {
    if let Some(server) = SERVER.get() {
        if let Some(req) = server.received_requests().await {
            assert!(req.len() > 0);
            let last_req = &req[req.len() - 1];
            let expected_method = Method::from_str(&request_type);
            match expected_method {
                Ok(method) => {
                    assert_eq!(last_req.method, method);
                }
                Err(e) => bail!(e),
            }
        }
    }

    // remove mocks
    if let Some(server) = SERVER.get() {
        server.reset().await;
    }
    Ok(())
}

#[then(expr = "the response should be of type {word}")]
async fn response_type_should_be(w: &mut ForgeWorld, expected: String) -> Result<()> {
    let snake_name = expected.to_case(convert_case::Case::Snake);
    let actual = get_type_name(w, &snake_name)?;
    assert!(actual.contains(&expected));
    Ok(())
}

#[then(expr = "the response should have a property {word} with value {word}")]
async fn response_should_have_property(
    w: &mut ForgeWorld,
    property: String,
    expected_value: String,
) -> Result<()> {
    if let Some(last_response) = &w.last_object_response {
        let serialized = &last_response.1;
        let dynamic_json = serde_json::from_str::<Value>(&serialized)?;
        let data_container = dynamic_json.get("data").context("cannot access data")?;
        let actual_value = data_container
            .get(&property)
            .context(format!("cannot access property {}", property))?;
        match expected_value.parse::<i32>() {
            Ok(nb) => {
                assert_eq!(&json!(nb), actual_value);
            }
            Err(_) => {
                assert_eq!(&json!(expected_value), actual_value);
            }
        }
    } else {
        panic!("no last response found");
    }
    Ok(())
}

#[then(expr = "it should generate a model object named {word}")]
async fn model_should_have_object(w: &mut ForgeWorld, expected: String) -> Result<()> {
    let snake_name = expected.to_case(convert_case::Case::Snake);
    let info = get_type_information(w, &snake_name)?;
    let actual = info.name;
    assert!(actual.contains(&expected));
    Ok(())
}

#[then(regex = r"(\S+) should have an? (\S+) property named (\S+) of type (\S+)")]
async fn object_should_have_type(
    w: &mut ForgeWorld,
    object: String,
    modifier: String,
    expected_name: String,
    expected_type: String,
) -> Result<()> {
    let snake_name = object.to_case(convert_case::Case::Snake);
    let info = get_type_information(w, &snake_name)?;
    let expected_in_snake_case = RString::from(expected_name.to_case(convert_case::Case::Snake));
    assert!(info.fields.contains_key(&expected_in_snake_case));
    let actual_type = info
        .fields
        .get(&expected_in_snake_case)
        .context("cannot get type from the map")?;
    match &expected_type[..] {
        "number" => {
            assert!(actual_type.contains("i32"));
        }
        "string" => {
            assert!(actual_type.contains("String"));
        }
        complex_type => {
            assert!(actual_type.contains(&complex_type));
        }
    };
    // Optional / Required check
    match &modifier[..] {
        "optional" => {
            assert!(actual_type.contains("Option<"));
        },
        "required" => {
            assert!(!actual_type.contains("Option<"));
        },
        _ => {
            panic!("unrecognised modifier");
        }

    }
    Ok(())
}
