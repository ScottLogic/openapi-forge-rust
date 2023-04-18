use anyhow::{Result, bail, Context};
use cucumber::{given, gherkin::Step};

use crate::{ForgeWorld, util::{hash_an_object, write_schema_to_file, forge, compile_generated_api}};

#[given(expr = "an API with the following specification")]
async fn given_api_specification(w: &mut ForgeWorld, step: &Step) -> Result<()> {
    // schema
    if let Some(spec) = step.docstring() {
        let hash = hash_an_object(spec);
        w.library_name_modifier = Some(hash);
        write_schema_to_file(spec, w.library_name_modifier.context("library modifier")?)
            .await?;
    } else {
        bail!("API spec not found");
    }
    // forge + compile + set
    forge(w.library_name_modifier.context("library modifier")?).await?;
    compile_generated_api(w.library_name_modifier.context("library modifier")?).await?;
    w.set_library()?;
    w.set_reset_client(None)?;
    Ok(())
}