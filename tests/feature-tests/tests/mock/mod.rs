use anyhow::{ Context, Result };
use wiremock::{ matchers, Mock, MockServer, ResponseTemplate };

use once_cell::sync::OnceCell;

static SERVER: OnceCell<MockServer> = OnceCell::new();

pub const PORT: u16 = 8888;

pub struct ForgeMockServer;

impl ForgeMockServer {
    pub async fn init_mock_server() -> Result<()> {
        let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;
        let mock_server = MockServer::builder().listener(listener).start().await;
        SERVER.get_or_init(|| mock_server);
        Ok(())
    }

    pub fn get_server() -> Result<&'static MockServer> {
        SERVER.get().context("mock server is not initialized")
    }

    pub async fn set_mock_with_string_response(response: &str) -> Result<()> {
        ForgeMockServer::reset_server().await?;
        let server = ForgeMockServer::get_server()?;
        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(200).set_body_string(response))
            .expect(1)
            .mount(server).await;

        Ok(())
    }

    pub async fn set_mock_with_json_response(raw_response: &str) -> Result<()> {
        ForgeMockServer::reset_server().await?;
        let server = ForgeMockServer::get_server()?;
        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(200).set_body_raw(raw_response, "application/json"))
            .expect(1)
            .mount(server).await;

        Ok(())
    }

    pub async fn set_mock_empty() -> Result<()> {
        ForgeMockServer::reset_server().await?;
        let server = ForgeMockServer::get_server()?;
        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(server).await;

        Ok(())
    }

    pub async fn set_mock_with_header(header: (&str, &str)) -> Result<()> {
        ForgeMockServer::reset_server().await?;
        let server = ForgeMockServer::get_server()?;
        Mock::given(matchers::any())
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header(header.0, header.1)
                    .set_body_raw("{}".as_bytes(), "application/json")
            )
            .expect(1)
            .mount(server).await;

        Ok(())
    }

    pub async fn reset_server() -> Result<()> {
        let server = ForgeMockServer::get_server()?;
        server.reset().await;
        Ok(())
    }
}