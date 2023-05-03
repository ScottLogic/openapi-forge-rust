use anyhow::{ Ok, Result };
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use std::path::Path;
use tokio::fs::{ DirBuilder, File, OpenOptions };
use tokio::io::{ AsyncBufReadExt, AsyncWriteExt, BufReader };
use tokio::process::Command;

pub(crate) const FEATURES: &str = "tests/features";

const GENERATED_API_PARENT: &str = ".generated-apis";

#[cfg(unix)]
pub(crate) mod platform {
    pub(crate) const PREFIX: &str = "lib";
    pub(crate) const EXTENSION: &str = "so";
}

#[cfg(windows)]
pub(crate) mod platform {
    pub(crate) const PREFIX: &str = "";
    pub(crate) const EXTENSION: &str = "dll";
}

macro_rules! get_schema_path {
    ($expression:expr) => {
        format!("{}/schema_{}.json", GENERATED_API_PARENT, $expression)
    };
}

macro_rules! get_generated_api_path {
    ($expression:expr) => {
        format!("{}/project_{}", GENERATED_API_PARENT, $expression)
    };
}

macro_rules! get_generated_shared_object_path {
    ($expression:expr) => {
        format!(
            "target/debug/{}openapi_forge_project_{}.{}",
            crate::util::platform::PREFIX,
            $expression,
            crate::util::platform::EXTENSION
        )
    };
}

pub(crate) use get_generated_shared_object_path;

pub async fn write_schema_to_file(contents: &str, file_name_modifier: u64) -> Result<()> {
    let mut file = File::create(get_schema_path!(file_name_modifier)).await?;
    file.write_all(contents.as_bytes()).await?;
    Ok(())
}

pub async fn clean_up_all() -> Result<()> {
    let _ret = tokio::fs::remove_dir_all(GENERATED_API_PARENT).await;
    let _ret = tokio::fs::remove_dir_all(FEATURES).await;
    Ok(())
}

pub async fn forge(modifier: u64) -> Result<()> {
    let mut forge_process = Command::new("node")
        .arg("../../../openapi-forge/src/index.js")
        .arg("forge")
        .arg(get_schema_path!(modifier))
        .arg("../../")
        .arg("-o")
        .arg(get_generated_api_path!(modifier))
        .arg("--logLevel")
        .arg("quiet")
        .arg("--generator.cabi_testing")
        .arg("true")
        .stdout(std::process::Stdio::null())
        .spawn()?;
    let _status = forge_process.wait().await?;
    Ok(())
}

pub async fn copy_feature_files() -> Result<()> {
    let mut entries = tokio::fs::read_dir("../../../openapi-forge/features/").await?;
    while let Some(entry) = entries.next_entry().await? {
        tokio::fs::copy(entry.path(), Path::new(FEATURES).join(entry.file_name())).await?;
    }
    Ok(())
}

pub async fn compile_generated_api(modifier: u64) -> Result<()> {
    // change project name first
    change_project_name(modifier).await?;
    // compile
    let mut compile_proceess = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(get_generated_api_path!(modifier) + "/Cargo.toml")
        .arg("--quiet")
        .stdout(std::process::Stdio::null())
        .spawn()?;
    let _status = compile_proceess.wait().await?;
    Ok(())
}

pub fn hash_an_object<T>(obj: T) -> u64 where T: Hash {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

// The reason this fn exists is that, if the .so file-name is shared, the library import creates issues and fetches the old one from the memory
// Also using different .so files allows concurrent api generation.
// To do that, instead of bringing toml and serde crate dependencies for testing, we do it manually just for one line change
pub async fn change_project_name(modifier: u64) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .open(get_generated_api_path!(modifier) + "/Cargo.toml").await?;
    let mut line_reader = BufReader::new(file).lines();
    let mut contents = vec![];
    while let Some(line) = line_reader.next_line().await? {
        contents.push(line);
    }
    // open again
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(get_generated_api_path!(modifier) + "/Cargo.toml").await?;
    contents[1] = format!("name = \"openapi_forge_project_{}\"", modifier);
    // truncate
    file.set_len(0).await?;
    // write
    file.write_all(contents.join("\n").as_bytes()).await?;
    Ok(())
}

pub async fn create_project_folders() -> Result<()> {
    DirBuilder::new().recursive(true).create(GENERATED_API_PARENT).await?;
    DirBuilder::new().recursive(true).create(FEATURES).await?;
    Ok(())
}