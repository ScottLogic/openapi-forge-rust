use anyhow::Result;
use convert_case::Casing;
use cucumber::{ gherkin::Step, given, then, when };
use url::Url;
use wiremock::{ matchers::any, Mock, ResponseTemplate };

use crate::SERVER;
use crate::{ ffi::run_method_no_params, util, ForgeWorld };

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
            .respond_with(
                ResponseTemplate::new(200).set_body_json({
                    let a = format!("{{'a':'{}'}}", method_name);
                    a
                })
            )
            .expect(1)
            .mount(server).await;
    }
    // run method
    run_method_no_params(w, &method_name)?;
    Ok(())
}

#[then(expr = "the requested URL should be {word}")]
async fn requested(_w: &mut ForgeWorld, url: String) -> Result<()> {
    if let Some(server) = SERVER.get() {
        if let Some(req) = server.received_requests().await {
            assert!(req.len() > 0);
            let last_req = &req[req.len() - 1];
            let expected_url = Url::parse(&url)?;
            let actual_url = &last_req.url;
            // only check the path since we do full http mock
            assert_eq!(expected_url.path(), actual_url.path());
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