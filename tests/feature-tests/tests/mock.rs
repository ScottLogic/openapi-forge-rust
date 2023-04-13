use wiremock::MockServer;

use once_cell::sync::OnceCell;

pub static SERVER: OnceCell<MockServer> = OnceCell::new();

pub const PORT: u16 = 8888;

pub async fn init_mock_server(port: u16) -> anyhow::Result<()> {
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", port))?;
    let mock_server = MockServer::builder().listener(listener).start().await;
    SERVER.get_or_init(|| mock_server);
    Ok(())
}
