use anyhow::Result;
use convert_case::Casing;
use cucumber::{given, gherkin::Step, when, then};
use url::Url;
use wiremock::{Mock, matchers::any, ResponseTemplate};

use crate::SERVER;
use crate::{ForgeWorld, util, api::run_method_no_params};


#[given(expr = "an API with the following specification")]
async fn api_specification(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    if let Some(spec) = step.docstring() {
        util::write_schema_to_file(spec).await?;
    }
    util::forge().await?;
    util::compile_generated_api().await?;
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
            .respond_with(ResponseTemplate::new(200).set_body_json("{'a':'a'}"))
            .expect(1)
            .mount(server)
            .await;
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
            let last_req = &req[ req.len() - 1 ];
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
