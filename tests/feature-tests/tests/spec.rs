use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

use abi_stable::std_types::{ROption, RString};
use anyhow::{Result, bail};
use convert_case::Casing;
use cucumber::{gherkin::Step, given, then, when};
use url::Url;
use wiremock::http::Method;
use wiremock::{matchers::any, Mock, ResponseTemplate};

use crate::ffi::{run_method_one_param, run_method_two_params};
use crate::SERVER;
use crate::{ffi::run_method_no_params, util, ForgeWorld};

#[given(expr = "an API with the following specification")]
async fn api_specification(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    // schema
    if let Some(spec) = step.docstring() {
        let hash = util::hash_an_object(spec);
        w.library_name_modifier = Some(hash);
        util::write_schema_to_file(spec, w.library_name_modifier.unwrap()).await?;
    }
    // forge + compile + set
    util::forge(w.library_name_modifier.unwrap()).await?;
    util::compile_generated_api(w.library_name_modifier.unwrap()).await?;
    // maybe move to another location
    w.set_library()?;
    w.set_reset_client(None)?;
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
    run_method_no_params(w, &method_name)?;
    Ok(())
}

#[when(expr = "calling the spied method {word} without params")]
async fn call_spied_method_without_params(w: &mut ForgeWorld, method_name: String) -> Result<()> {
    call_method_without_params(w, method_name).await?;
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
async fn requested_type(_w: &mut ForgeWorld, request_type: String) -> Result<()> {
    if let Some(server) = SERVER.get() {
        if let Some(req) = server.received_requests().await {
            assert!(req.len() > 0);
            let last_req = &req[req.len() - 1];
            let expected_method = Method::from_str(&request_type);
            match expected_method {
                Ok(method) => { 
                    assert_eq!(last_req.method, method); 
                },
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

// calling the method getThings with parameters
