mod util;

use std::marker::PhantomData;

use anyhow::{Ok, Result};
use convert_case::Casing;
use cucumber::{gherkin::Step, given, when, World};
use http::Response;
use libloading::Library;
use mockito::Server;
use util::{load_client, load_config, load_http_client, load_method_test};

#[derive(Debug)]
#[repr(C)]
pub struct ApiClient {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Configuration {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Client {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct ForgeResponse<T> {
    _private: [u8; 0],
    _phantom: PhantomData<T>,
}

#[derive(Debug, World)]
// Accepts both sync/async and fallible/infallible functions.
#[world(init = Self::new)]
pub struct ForgeWorld {
    server: Server,
    library: Option<Library>,
    config: Option<Box<Configuration>>,
    http_client: Option<Box<Client>>,
    api_client: Option<Box<ApiClient>>,
}

impl ForgeWorld {
    async fn new() -> Self {
        Self {
            server: Server::new_with_port_async(5000).await,
            library: None,
            config: None,
            http_client: None,
            api_client: None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    util::clean_up().await?;
    ForgeWorld::run("tests/features/").await;
    Ok(())
}

#[given(expr = "an API with the following specification")]
async fn api_specification(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    if let Some(spec) = step.docstring() {
        util::write_schema_to_file(spec).await?;
    }
    util::forge().await?;
    util::compile_generated_api().await?;
    let lib = util::load_generated_api()?;
    w.library = Some(lib);
    let config = load_config(w)?;
    w.config = Some(config);
    let http_client = load_http_client(w)?;
    w.http_client = Some(http_client);
    let client = load_client(w)?;
    w.api_client = Some(client);
    Ok(())
}

#[when(expr = "calling the method {word} without params")]
async fn call_method_without_params(w: &mut ForgeWorld, method_name: String) -> Result<()> {
    let method_name = method_name.to_case(convert_case::Case::Snake);
    let mock = w.server.mock("GET", "/e1").with_body_from_request(|req| {
        assert_eq!(req.path(), "127.0.0.1/e1");
        let mut res = Response::builder().status(http::StatusCode::OK).body("aaa".to_owned())?;
        res.into_body()
    });
    mock.create_async().await;
    load_method_test(w, &method_name).await?;

    Ok(())
}
