use abi_stable::std_types::RString;
use anyhow::{Ok, Result};
use libloading::Library;
use tokio::fs::{remove_file, File, remove_dir_all};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::{ForgeWorld, Configuration, Client, ApiClient, ForgeResponse};

const SCHEMA_PATH: &str = "schema.json";
const GENERATED_API_PATH: &str = "generated-api"; 
const GENERATED_SHARED_OBJECT_PATH: &str = "target/debug/libopenapi_forge_project.so";

pub async fn write_schema_to_file(contents: &str) -> Result<()> {
    let mut file = File::create(SCHEMA_PATH).await?;
    file.write_all(contents.as_bytes()).await?;
    Ok(())
}

pub async fn clean_up() -> Result<()> {
    // Ignore file not found errors
    let _ = remove_file(SCHEMA_PATH).await;
    let _ = remove_file(GENERATED_SHARED_OBJECT_PATH).await;
    let _ = remove_dir_all(GENERATED_API_PATH).await;
    Ok(())
}

pub async fn forge() -> Result<()> {
    let mut forge_process = Command::new("node")
        .arg("../../../openapi-forge/src/index.js")
        .arg("forge")
        .arg(SCHEMA_PATH)
        .arg("../../")
        .arg("-o")
        .arg(GENERATED_API_PATH)
        .arg("--logLevel")
        .arg("quiet")
        .arg("--generator.cabi_testing")
        .arg("true")
        .spawn()?;
    let _status = forge_process.wait().await?;
    println!("the command exited with: {}", _status);
    Ok(())
}

pub async fn compile_generated_api() -> Result<()> {
    let mut compile_proceess = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg("generated-api/Cargo.toml")
        .arg("--quiet")
        .spawn()?;
    let _status = compile_proceess.wait().await?;
    println!("the command exited with: {}", _status);
    Ok(())
}

pub fn load_generated_api() -> Result<Library> {
    // SAFETY
    // This call should be always followed after the generated api is compiled.
    unsafe {
        let lib = libloading::Library::new(GENERATED_SHARED_OBJECT_PATH)?;
        Ok(lib)
    }
}

pub fn load_config(w: &mut ForgeWorld) -> Result<Box<Configuration>> {
    unsafe {
        if let Some(library) = &w.library {
            let func: libloading::Symbol<extern fn(RString) -> Box<Configuration>> = library.get(b"c_config_new")?;
            let c = func("http://127.0.0.1:5000".into());
            Ok(c)
        }
        else {
            panic!("a")
        }
    }
}

pub fn load_http_client(w: &mut ForgeWorld) -> Result<Box<Client>> {
    unsafe {
        if let Some(library) = &w.library {
            let func: libloading::Symbol<extern fn() -> Box<Client>> = library.get(b"c_reqwest_client_new")?;
            let c = func();
            Ok(c)
        }
        else {
            panic!("a")
        }
    }
}

pub fn load_client(w: &mut ForgeWorld) -> Result<Box<ApiClient>> {
    unsafe {
        let config = w.config.take();
        let client = w.http_client.take();
        if let Some(library) = &w.library {
            let func: libloading::Symbol<extern fn(Box<Configuration>, Box<Client>) -> Box<ApiClient>> = library.get(b"c_api_client_new")?;
            let c = func(config.unwrap(), client.unwrap());
            Ok(c)
        }
        else {
            panic!("a")
        }
    }
}

pub async fn load_method_test(w: &mut ForgeWorld, method_name: &str) -> Result<()> {
    unsafe{
        let c_method = format!("c_api_client_{}", method_name);
        let c_method_bytes = c_method.as_bytes();
        if let Some(library) = &w.library {
            dbg!(&c_method);
            let func: libloading::Symbol<extern fn(Box<ApiClient>) -> Box<ForgeResponse<String>>> = library.get(c_method_bytes)?;
            let api_client = w.api_client.take();
            dbg!("asd");
            let r = func(api_client.unwrap());
        }
    }
    Ok(())
}