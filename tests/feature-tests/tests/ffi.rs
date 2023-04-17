use abi_stable::std_types::{RString, ROption};
use anyhow::Result;
use libloading::Library;

use crate::mock::PORT;
use crate::{ data::{ ApiClient, Client, Configuration, ForgeResponse }, ForgeWorld };

pub fn get_generated_library(hash: u64) -> Result<Library> {
    // SAFETY
    // This call should be always followed after the generated api is compiled.
    unsafe {
        let lib = libloading::Library::new(crate::util::get_generated_shared_object_path!(hash))?;
        Ok(lib)
    }
}

pub fn get_config(w: &mut ForgeWorld) -> Result<Box<Configuration>> {
    unsafe {
        if let Some(library) = &w.library {
            let func: libloading::Symbol<
                extern "C" fn(RString) -> Box<Configuration>
            > = library.get(b"c_config_new")?;
            let c = func(format!("http://127.0.0.1:{}", PORT).into());
            Ok(c)
        } else {
            panic!("get_config")
        }
    }
}

pub fn run_config_idx_change(w: &mut ForgeWorld, idx: u8) -> Result<()> {
    unsafe {
        if let Some(library) = &w.library {
            let config = w.config.take();
            let func: libloading::Symbol<
                extern "C" fn(Box<Configuration>, u8) -> Box<Configuration>
            > = library.get(b"c_config_select_server_index")?;
            if let Some(config) = config {
                let new_config = func(config, idx);
                w.config = Some(new_config);
                Ok(())
            } else {
                panic!("run_config_idx_change cfg")
            }
        } else {
            panic!("run_config_idx_change")
        }
    }
}

pub fn get_http_client(w: &mut ForgeWorld) -> Result<Box<Client>> {
    unsafe {
        if let Some(library) = &w.library {
            let func: libloading::Symbol<extern "C" fn() -> Box<Client>> = library.get(
                b"c_reqwest_client_new"
            )?;
            let c = func();
            Ok(c)
        } else {
            panic!("get_http_client")
        }
    }
}

pub fn get_api_client(w: &mut ForgeWorld) -> Result<Box<ApiClient>> {
    unsafe {
        let config = w.config.take();
        let client = w.http_client.take();
        if let Some(library) = &w.library {
            let func: libloading::Symbol<
                extern "C" fn(Box<Configuration>, Box<Client>) -> Box<ApiClient>
            > = library.get(b"c_api_client_new")?;
            match (config, client) {
                (Some(config), Some(client)) => {
                    let api_client = func(config, client);
                    Ok(api_client)
                }
                _ => panic!("get_api_client cfg"),
            }
        } else {
            panic!("get_api_client")
        }
    }
}

// if api client is not used and we will be generating a new one, don't drop the old one here
// ask generated library to drop it - only the library knows the memory layout.
pub fn drop_api_client_if_exists(w: &mut ForgeWorld) -> Result<()> {
    unsafe {
        let api_client = w.api_client.take();
        if let Some(api_client) = api_client {
            if let Some(library) = &w.library {
                let func: libloading::Symbol<extern "C" fn(Box<ApiClient>)> = library.get(
                    b"c_api_client_drop"
                )?;
                func(api_client);
            } else {
                panic!("get_api_client");
            }
        }
        Ok(())
    }
}


pub fn run_method_no_params_with_return<T>(
    w: &mut ForgeWorld,
    method_name: &str
) -> Result<Box<ForgeResponse<T>>> {
    unsafe {
        let c_method = format!("c_api_client_{}", method_name);
        let c_method_bytes = c_method.as_bytes();
        if let Some(library) = &w.library {
            let func: libloading::Symbol<
                extern "C" fn(Box<ApiClient>) -> Box<ForgeResponse<T>>
            > = library.get(c_method_bytes)?;
            let api_client = w.api_client.take();
            if let Some(api_client) = api_client {
                let ret = func(api_client);
                Ok(ret)
            } else {
                panic!("run_method_no_params api_client")
            }
        } else {
            panic!("run_method_no_params")
        }
    }
}

pub fn run_method_one_param<T>(
    w: &mut ForgeWorld,
    method_name: &str,
    arg_1: ROption<T>,
) -> Result<Box<ForgeResponse<RString>>> {
    unsafe {
        let c_method = format!("c_api_client_{}", method_name);
        let c_method_bytes = c_method.as_bytes();
        if let Some(library) = &w.library {
            let func: libloading::Symbol<
                extern "C" fn(Box<ApiClient>, ROption<T>) -> Box<ForgeResponse<RString>>
            > = library.get(c_method_bytes)?;
            let api_client = w.api_client.take();
            if let Some(api_client) = api_client {
                let ret = func(api_client, arg_1);
                Ok(ret)
            } else {
                panic!("run_method_no_params api_client")
            }
        } else {
            panic!("run_method_no_params")
        }
    }
}

pub fn run_method_two_params<T: std::fmt::Debug, U: std::fmt::Debug>(
    w: &mut ForgeWorld,
    method_name: &str,
    arg_1: ROption<T>,
    arg_2: ROption<U>,
) -> Result<Box<ForgeResponse<RString>>> {
    unsafe {
        let c_method = format!("c_api_client_{}", method_name);
        let c_method_bytes = c_method.as_bytes();
        if let Some(library) = &w.library {
            let func: libloading::Symbol<
                extern "C" fn(Box<ApiClient>, ROption<T>, ROption<U>) -> Box<ForgeResponse<RString>>
            > = library.get(c_method_bytes)?;
            let api_client = w.api_client.take();
            if let Some(api_client) = api_client {
                let ret = func(api_client, arg_1, arg_2);
                Ok(ret)
            } else {
                panic!("run_method_no_params api_client")
            }
        } else {
            panic!("run_method_no_params")
        }
    }
}