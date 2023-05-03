## OpenAPI Forge - Rust

This repository is the Rust generator for the [OpenAPI Forge](https://github.com/ScottLogic/openapi-forge), see that repository for usage instructions:

## Configuration

OpenAPI Forge is opinionated in its approach, we don't provide a vast range of configuration options, just the essentials! You can list the generator-specific configuration options by running the `generate-options` command as follows:

```
% openapi-forge generator-options openapi-forge-rust
This generator has a number of additional options which can be supplied when executing the 'forge' command.

Options:
  --generator.package <value>  The package for the generated classes.
```

## Development

### Running

To run this generator, you also need to have [OpenAPI Forge](https://github.com/ScottLogic/openapi-forge) installed, or the repository checked out. Assuming you have it installed as a global module, you can run this generator as follows:

```
$ openapi-forge forge
 \ https://petstore3.swagger.io/api/v3/openapi.json
 \ .
 \ -o api
```

This generates an API from the Pet Store swagger definition, using the generator within the current folder (`.`), outputting the results to the `api` folder.

Afterwards, by modiyfing the main.rs in the generated api project, we can fetch every available pet and print them into stdout.

```rust
use anyhow::Result;
use api_client::api_client_pet::ApiClientPet;
use config::Configuration;
use reqwest::Client;

#[tokio::main]
pub async fn main() -> Result<()> {
  let config = Configuration::new("https://petstore3.swagger.io");
  let client = Client::new();
  let api_client_pet = ApiClientPet::new(config, client);
  let response = api_client_pet.find_pets_by_status(
    Some("available".into())
  ).await?;
  dbg!(&response);
  Ok(())
}
```

### Testing

The standard test script is used to execute the BDD-style tests against this generator.

```
npm run test
```

The script expects that the openapi-forge project (which is where the BDD feature files are located) is checked out at the same folder-level as this project. In-depth documentation can be found at [features/README.md](features/README.md).

### Linting

Two scripts are available to help you find linting errors:

```
npm run lint:check:all
```

This runs eslint in check mode which will raise errors found but not try and fix them. This is also ran on a PR and a push to main. It will fail if any errors were found.

```
npm run lint:write:all
```

This runs eslint in write mode which will raise errors found and try to fix them.
