```rust
#[tokio::main]
pub async fn main() {
    let config = Configuration::new("https://petstore3.swagger.io");
    let api_client_pet = ApiClientPet::new(config);
    let response = api_client_pet.findPetsByStatus(None).await;
    dbg!(response);
}
```