use std::{ collections::HashSet, path::Path, str::FromStr };

use abi_stable::std_types::RString;
use anyhow::{ bail, Context, Result };
use chrono::{ naive::NaiveDate, DateTime, Utc };
use convert_case::Casing;
use cucumber::then;
use serde_json::{ json, Value };
use url::Url;
use wiremock::http::{ HeaderName, Method };

use crate::{ ffi::call::FFICaller, mock::ForgeMockServer, ForgeWorld };

#[then(expr = "the requested URL should be {word}")]
async fn requested_url(_w: &mut ForgeWorld, url: String) -> Result<()> {
    let server = ForgeMockServer::get_server()?;
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
    } else {
        bail!("Problem with received requests");
    }
    Ok(())
}

#[then(expr = "the request method should be of type {word}")]
async fn requested_type_should_be(_w: &mut ForgeWorld, request_type: String) -> Result<()> {
    let server = ForgeMockServer::get_server()?;
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
    } else {
        bail!("Problem with received requests");
    }
    Ok(())
}

#[then(expr = "the request should have a header property with value {word}")]
async fn request_should_have_header(_w: &mut ForgeWorld, value: String) -> Result<()> {
    let server = ForgeMockServer::get_server()?;
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
    } else {
        bail!("Problem with received requests");
    }
    Ok(())
}

#[then(expr = "the response should be of type {word}")]
async fn response_type_should_be(w: &mut ForgeWorld, expected: String) -> Result<()> {
    if let Some(last_call) = &w.last_fn_call_sign {
        assert_eq!(&last_call.return_type, &expected);
    } else {
        bail!("no last call");
    }
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
        let mut dynamic_json = serde_json::from_str::<Value>(&serialized)?;
        let data_container = dynamic_json.get_mut("data").context("cannot access data")?;
        let actual_value = data_container
            .get_mut(&property)
            .context(format!("cannot access property {}", property))?
            .take();
        let possible_date_time = DateTime::parse_from_rfc3339(&expected_value);
        let possible_naive_date = NaiveDate::parse_from_str(&expected_value, "%Y-%m-%d");
        let possible_nb = expected_value.parse::<i64>();
        match (possible_date_time, possible_naive_date, possible_nb) {
            (Ok(date_time), Err(_), Err(_)) => {
                let actual_date_time = serde_json::from_value::<DateTime<Utc>>(actual_value)?;
                assert_eq!(date_time, actual_date_time);
            }
            (Err(_), Ok(naive_date), Err(_)) => {
                let actual_date = serde_json::from_value::<NaiveDate>(actual_value)?;
                assert_eq!(naive_date, actual_date);
            }
            (Err(_), Err(_), Ok(nb)) => {
                let actual_nb = actual_value.as_i64().context("cannot convert actual value to nb")?;
                assert_eq!(nb, actual_nb);
            }
            (Err(_), Err(_), Err(_)) => {
                assert_eq!(json!(expected_value), actual_value);
            }
            _ => {
                bail!("Other cases are not possible");
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
    let info = FFICaller::model_get_type_information(w, &snake_name)?;
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
    let info = FFICaller::model_get_type_information(w, &snake_name)?;
    let expected_name_snake_case = RString::from(expected_name.to_case(convert_case::Case::Snake));
    assert!(info.fields.contains_key(&expected_name_snake_case));
    let actual_type = info.fields
        .get(&expected_name_snake_case)
        .context("cannot get type from the map")?;
    match &expected_type[..] {
        "number" => {
            assert!(actual_type.contains("i64"));
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
    let server = ForgeMockServer::get_server()?;
    if let Some(req) = server.received_requests().await {
        assert!(req.len() > 0);
        let last_req = &req[req.len() - 1];
        assert_eq!(last_req.body, body.as_bytes());
    } else {
        bail!("Problem with received requests");
    }
    Ok(())
}

#[then(regex = r"the request header should have a cookie property with value (\S+)")]
async fn request_header_should_have_cookie(_w: &mut ForgeWorld, cookie_str: String) -> Result<()> {
    let server = ForgeMockServer::get_server()?;
    if let Some(req) = server.received_requests().await {
        assert!(req.len() > 0);
        let last_req = &req[req.len() - 1];
        if let Some(cookie_value) = last_req.headers.get(&HeaderName::from("cookie")) {
            assert!(cookie_value.iter().any(|h| *h == cookie_str));
        } else {
            bail!("no cookie");
        }
    } else {
        bail!("Problem with received requests");
    }
    Ok(())
}

#[then(regex = r"the response should be equal to (.+)")]
async fn response_should_be_equal_to(w: &mut ForgeWorld, expected: String) -> Result<()> {
    if let Some(actual) = &w.last_string_response {
        assert_eq!(actual, &expected);
    } else {
        bail!("no response string");
    }
    Ok(())
}

#[then(expr = "the response should be an array")]
async fn response_should_be_an_array(w: &mut ForgeWorld) -> Result<()> {
    if let Some(last) = &w.last_object_response {
        let serialized = &last.1;
        let value = serde_json::from_str::<Value>(serialized.as_str())?;
        let data = value.get("data").context("data contrainer")?;
        assert!(data.is_array());
    } else {
        bail!("no last response");
    }
    Ok(())
}

#[then(expr = "the response should have a header {word} with value {word}")]
async fn response_should_have_header(
    w: &mut ForgeWorld,
    name: String,
    value: String
) -> Result<()> {
    if let Some(last_response) = &w.last_object_response {
        let json_value = serde_json::from_str::<Value>(&last_response.1)?;
        let headers = json_value.get("headers").context("no headers")?;
        let header_object = headers.as_object().context("object")?;
        let actual = header_object
            .get(&name)
            .context("no header name")?
            .as_str()
            .context("cannot str")?;
        assert_eq!(actual, &value);
    } else {
        bail!("no last response");
    }
    Ok(())
}

#[then(expr = "the response should be null")]
async fn response_should_be_null(w: &mut ForgeWorld) -> Result<()> {
    if let Some(last_response) = &w.last_object_response {
        let value = serde_json::from_str::<Value>(&last_response.1)?;
        let data = value.get("data").context("no data")?;
        assert!(data.is_null());
    } else {
        bail!("No last response");
    }
    Ok(())
}

#[then(expr = "the api file with tag {word} exists")]
async fn api_client_with_tag_exists(w: &mut ForgeWorld, tag: String) -> Result<()> {
    let tag = &tag[1..tag.len() - 1];
    let some_tag = if tag.is_empty() { None } else { Some(tag) };
    w.set_reset_client(None, some_tag)?;
    Ok(())
}

#[then(expr = "the api file with tag {word} does not exist")]
async fn api_client_tag_do_not_exist(w: &mut ForgeWorld, tag: String) -> Result<()> {
    let tag = &tag[1..tag.len() - 1];
    let some_tag = if tag.is_empty() { None } else { Some(tag) };
    if let Err(_e) = w.set_reset_client(None, some_tag) {
        Ok(())
    } else {
        bail!("The client exists");
    }
}

#[then(expr = "the method {word} should be present in the api file with tag {word}")]
async fn api_client_with_tag_should_have_method(
    w: &mut ForgeWorld,
    method_name: String,
    tag: String
) -> Result<()> {
    let tag = &tag[1..tag.len() - 1];
    let some_tag = if tag.is_empty() { None } else { Some(tag) };
    let method_name = &method_name[1..&method_name.len() - 1];
    w.set_reset_client(None, some_tag)?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    let method_name_snake = method_name.to_case(convert_case::Case::Snake);
    FFICaller::check_method_exists(w, &api_client_name, &method_name_snake)?;
    Ok(())
}

#[then(expr = "the method {word} should not be present in the api file with tag {word}")]
async fn api_client_with_tag_should_not_have_method(
    w: &mut ForgeWorld,
    method_name: String,
    tag: String
) -> Result<()> {
    let tag = &tag[1..tag.len() - 1];
    let some_tag = if tag.is_empty() { None } else { Some(tag) };
    let method_name = &method_name[1..&method_name.len() - 1];
    w.set_reset_client(None, some_tag)?;
    let api_client_name = w.api_client_name.take().context("No client name")?;
    let method_name_snake = method_name.to_case(convert_case::Case::Snake);
    if let Err(_e) = FFICaller::check_method_exists(w, &api_client_name, &method_name_snake) {
        Ok(())
    } else {
        bail!("The method exists");
    }
}