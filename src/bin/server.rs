use redust::{server, DEFAULT_HOST, DEFAULT_PORT};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener: TcpListener =
        TcpListener::bind(&format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT)).await?;
    println!("Server listening on {}:{}", DEFAULT_HOST, DEFAULT_PORT);

    let _ = server::run(listener).await;

    Ok(())
}
