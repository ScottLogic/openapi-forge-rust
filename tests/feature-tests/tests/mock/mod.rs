use anyhow::{ Result, bail };
use wiremock::{ MockServer, Mock, matchers, ResponseTemplate };

use once_cell::sync::OnceCell;

pub static SERVER: OnceCell<MockServer> = OnceCell::new();

pub const PORT: u16 = 8888;

pub async fn init_mock_server(port: u16) -> Result<()> {
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", port))?;
    let mock_server = MockServer::builder().listener(listener).start().await;
    SERVER.get_or_init(|| mock_server);
    Ok(())
}

pub async fn set_mock_with_string_response(response: &str) -> Result<()> {
    if let Some(server) = SERVER.get() {
        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(200).set_body_string(response))
            .expect(1)
            .mount(server).await;
    } else {
        bail!("Mock server cannot be accessed");
    }
    Ok(())
}

pub async fn set_mock_with_json_response(raw_response: &str) -> Result<()> {
    if let Some(server) = SERVER.get() {
        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(200).set_body_raw(raw_response, "application/json"))
            .expect(1)
            .mount(server).await;
    } else {
        bail!("Mock server cannot be accessed");
    }
    Ok(())
}