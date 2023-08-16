use crate::{cmd::Command, connection::Connection};
use tokio::net::TcpListener;

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
#[derive(Debug)]
struct Listener {
    listener: TcpListener,
}

/// Per-connection handler. Reads requests from `connection` and applies the
/// commands to `db`.
#[derive(Debug)]
struct ConnectionHandler {
    connection: Connection,
}

impl Listener {
    /// Run the server
    ///
    /// Listen for inbound connections. For each inbound connection, spawn a
    /// tokio task to process that connection.
    ///
    /// Returns `Err` if accepting returns an error. This can happen for a
    /// number reasons. e.g. if the operating system has reached an
    /// internal limit for max number of sockets, `tokio::net::tcp::listener::TcpListener.accept()` will fail.
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Accept a new socket. The `accept` method internally attempts to
            // recover errors, so if an error occurs here, we should propagate it
            println!("Accepting Inbound Connections");
            let (socket, _) = self.listener.accept().await?;

            // Create the necessary per-connection handler
            let mut handler = ConnectionHandler {
                // Initialize the connection state. This allocates read/write
                // buffers, and to perform RESP (de)-serialization
                connection: Connection::new(socket),
            };

            // Spawn a new task to process the connection.
            tokio::spawn(async move {
                // Process the connection. If an error is encountered, print it.
                if let Err(err) = handler.run().await {
                    println!("Connection Error | {:?}", err);
                }
            });
        }
    }
}

impl ConnectionHandler {
    /// Process a single connection.
    ///
    /// Request frames are read from the socket and processed. Responses are
    /// written back to the socket.
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Returns when a valid frame has been read.
        let frame = self.connection.read_frame().await?.unwrap();

        // Convert the RespType into a command struct.
        // This will return an error if the frame is not a valid command.
        let cmd = Command::infer_command(frame)?;

        println!("{:?}", cmd);

        // Execute the command
        // The connection is passed into the execute function which allows the
        // concrete command to write the response directly to the connection stream
        let _ = cmd.execute(&mut self.connection).await;

        Ok(())
    }
}

pub async fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the listener state
    let mut server = Listener { listener };

    server.run().await?;

    Ok(())
}
