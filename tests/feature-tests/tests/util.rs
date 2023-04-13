use anyhow::{Ok, Result};
use tokio::fs::{remove_dir_all, remove_file, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub const SCHEMA_PATH: &str = "schema.json";
pub const GENERATED_API_PATH: &str = "generated-api";
pub const GENERATED_SHARED_OBJECT_PATH: &str = "target/debug/libopenapi_forge_project.so";

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
