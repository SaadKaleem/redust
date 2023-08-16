use redust::{server, DEFAULT_PORT};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener: TcpListener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    println!("Server listening on 127.0.0.1:{}", DEFAULT_PORT);
    

    let _ = server::run(listener).await;

    Ok(())
}
