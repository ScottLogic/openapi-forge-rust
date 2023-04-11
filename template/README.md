```rust
use api_client::api_client_pet::ApiClientPet;
use config::Configuration;
use reqwest::Client;

#[tokio::main]
pub async fn main() {
    let config = Configuration::new("https://petstore3.swagger.io");
    let client = Client::new();
    let api_client_pet = ApiClientPet::new(config, client);
    let response = api_client_pet.find_pets_by_status(None).await;
    dbg!(&response);
}
```
