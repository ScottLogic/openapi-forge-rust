use std::{ collections::HashSet, path::Path, str::FromStr };

use abi_stable::std_types::RString;
use anyhow::{ bail, Context, Result };
use convert_case::Casing;
use cucumber::then;
use serde_json::{ json, Value };
use url::Url;
use wiremock::http::{ Method };

use crate::{ ffi::{ model_get_type_information, model_get_type_name }, ForgeWorld };

use crate::SERVER;

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
    Ok(())
}

#[then(expr = "the request should have a header property with value {word}")]
async fn request_should_have_header(_w: &mut ForgeWorld, value: String) -> Result<()> {
    if let Some(server) = SERVER.get() {
        if let Some(req) = server.received_requests().await {
            assert!(req.len() > 0);
            let last_req = &req[req.len() - 1];
            let headers = &last_req.headers;
            let header_values = headers
                .values()
                .flatten()
                .map(|h| h.as_str())
                .collect::<Vec<_>>();
            assert!(header_values.contains(&&value[..]));
        }
    }
    Ok(())
}

#[then(expr = "the response should be of type {word}")]
async fn response_type_should_be(w: &mut ForgeWorld, expected: String) -> Result<()> {
    let snake_name = expected.to_case(convert_case::Case::Snake);
    let actual = model_get_type_name(w, &snake_name)?;
    assert!(actual.contains(&expected));
    Ok(())
}

#[then(expr = "the response should have a property {word} with value {word}")]
async fn response_should_have_property(
    w: &mut ForgeWorld,
    property: String,
    expected_value: String
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
        bail!("no last response found");
    }
    Ok(())
}

#[then(expr = "it should generate a model object named {word}")]
async fn model_should_have_object(w: &mut ForgeWorld, expected: String) -> Result<()> {
    let snake_name = expected.to_case(convert_case::Case::Snake);
    let info = model_get_type_information(w, &snake_name)?;
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
    expected_type: String
) -> Result<()> {
    let snake_name = object.to_case(convert_case::Case::Snake);
    let info = model_get_type_information(w, &snake_name)?;
    let expected_name_snake_case = RString::from(expected_name.to_case(convert_case::Case::Snake));
    assert!(info.fields.contains_key(&expected_name_snake_case));
    let actual_type = info.fields
        .get(&expected_name_snake_case)
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
    }
    // Optional / Required check
    match &modifier[..] {
        "optional" => {
            assert!(actual_type.contains("Option<"));
        }
        "required" => {
            assert!(!actual_type.contains("Option<"));
        }
        _ => {
            bail!("unrecognised modifier");
        }
    }
    Ok(())
}

#[then(regex = r"the request should have a body with value (\S+)")]
async fn request_should_have_body(_w: &mut ForgeWorld, body: String) -> Result<()> {
    if let Some(server) = SERVER.get() {
        if let Some(req) = server.received_requests().await {
            assert!(req.len() > 0);
            let last_req = &req[req.len() - 1];
            assert_eq!(String::from_utf8(last_req.body.clone())?, body);
        }
    }
    Ok(())
}