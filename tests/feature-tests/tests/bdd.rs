mod data;
mod ffi;
mod mock;
mod spec;
mod util;

use anyhow::{ Ok, Result, Context };

use cucumber::World;
use data::*;
use ffi::{
    drop_api_client_if_exists,
    get_api_client,
    get_config,
    get_http_client,
    run_config_idx_change,
};
use libloading::Library;
use mock::SERVER;

use crate::mock::PORT;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct ForgeWorld {
    library: Option<Library>,
    library_name_modifier: Option<u64>,
    config: Option<Box<Configuration>>,
    http_client: Option<Box<Client>>,
    api_client: Option<Box<ApiClient>>,
    last_object_response: Option<FFISafeTuple<FFIObject>>,
}

impl ForgeWorld {
    async fn new() -> Self {
        Self {
            library: None,
            library_name_modifier: None,
            config: None,
            http_client: None,
            api_client: None,
            last_object_response: None,
        }
    }

    fn set_library(&mut self) -> Result<()> {
        let lib = ffi::get_generated_library(
            self.library_name_modifier.context("library modifier")?
        )?;
        self.library = Some(lib);
        Ok(())
    }

    fn set_reset_client(&mut self, server_idx: Option<u8>) -> Result<()> {
        drop_api_client_if_exists(self)?;
        let config = get_config(self)?;
        self.config = Some(config);
        if let Some(idx) = server_idx {
            run_config_idx_change(self, idx)?;
        }
        let http_client = get_http_client(self)?;
        self.http_client = Some(http_client);
        let client = get_api_client(self)?;
        self.api_client = Some(client);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    util::create_project_parent_dir().await?;
    mock::init_mock_server(PORT).await?;
    ForgeWorld::cucumber().run("tests/features").await;
    util::clean_up_all().await?;
    Ok(())
}