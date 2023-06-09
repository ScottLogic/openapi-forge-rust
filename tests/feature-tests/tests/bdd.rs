mod data;
mod ffi;
mod mock;
mod spec;
mod util;

use abi_stable::std_types::RString;
use anyhow::{ Context, Ok, Result };

use convert_case::Casing;
use cucumber::World;
use data::*;
use ffi::call::FFICaller;
use libloading::Library;
use mock::ForgeMockServer;

use crate::util::FEATURES;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct ForgeWorld {
    library: Option<Library>,
    library_name_modifier: Option<u64>,
    config: Option<Box<Configuration>>,
    http_client: Option<Box<Client>>,
    api_client_name: Option<String>,
    api_client: Option<Box<ApiClient>>,
    last_string_response: Option<RString>,
    last_object_response: Option<FFISafeTuple<FFIObject>>,
    last_fn_call_sign: Option<FnSignatureInformation>,
}

impl ForgeWorld {
    async fn new() -> Self {
        Self {
            library: None,
            library_name_modifier: None,
            config: None,
            http_client: None,
            api_client_name: None,
            api_client: None,
            last_string_response: None,
            last_object_response: None,
            last_fn_call_sign: None,
        }
    }

    fn set_library(&mut self) -> Result<()> {
        let lib = FFICaller::get_generated_library(
            self.library_name_modifier.context("library modifier")?
        )?;
        self.library = Some(lib);
        Ok(())
    }

    fn set_reset_client(&mut self, server_idx: Option<u8>, tag: Option<&str>) -> Result<()> {
        let api_client_name = if let Some(tag) = tag {
            format!("api_client_{}", tag.to_case(convert_case::Case::Snake))
        } else {
            "api_client".to_owned()
        };
        FFICaller::drop_api_client_if_exists(self, &api_client_name)?;
        let config = FFICaller::get_config(self)?;
        self.config = Some(config);
        if let Some(idx) = server_idx {
            FFICaller::run_config_idx_change(self, idx)?;
        }
        let http_client = FFICaller::get_http_client(self)?;
        self.http_client = Some(http_client);
        if let Some(tag) = tag {
            self.api_client_name = Some(
                format!("api_client_{}", tag.to_case(convert_case::Case::Snake))
            );
        } else {
            self.api_client_name = Some("api_client".to_owned());
        }
        let api_client = FFICaller::get_api_client(self, &api_client_name)?;
        self.api_client = Some(api_client);
        self.api_client_name = Some(api_client_name);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    util::create_project_folders().await?;
    util::copy_feature_files().await?;
    ForgeMockServer::init_mock_server().await?;
    ForgeWorld::cucumber().run(FEATURES).await;
    util::clean_up_all().await?;
    Ok(())
}