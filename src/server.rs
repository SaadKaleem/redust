use crate::{cmd::Command, Connection, ConnectionBase, RESPType, SharedStore};
use tokio::net::TcpListener;

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
#[derive(Debug)]
struct Listener {
    listener: TcpListener,

    /// Holds the data store around an `Arc`
    /// This is shared across each `Handler`
    shared_store: SharedStore,
}

/// Per-connection handler. Reads requests from `Connection`
/// and passes the `SharedStore` to the individual commands.
#[derive(Debug)]
struct ConnectionHandler {
    connection: Connection,
    shared_store: SharedStore,
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
            let (socket, _) = self.listener.accept().await?;

            // Create the necessary per-connection handler
            let mut handler = ConnectionHandler {
                // Initialize the connection state. This allocates read/write
                // buffers, and to perform RESP (de)-serialization
                connection: Connection::new(socket),

                // Get the shared data store. Internally, this is an
                // `Arc`, so a clone only increments the reference count.
                shared_store: self.shared_store.clone(),
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
        loop {
            let frame: RESPType;

            match self.connection.read_frame().await {
                Ok(Some(val)) => frame = val,
                Ok(None) => return Ok(()),
                Err(err) => return Err(err),
            }

            // Convert the RespType into a command struct.
            // This will return an error if the frame is not a valid command.
            match Command::infer_command(frame) {
                Ok(cmd) => {
                    // Execute the command
                    // The connection is passed into the execute function which allows the
                    // concrete command to write the response directly to the connection stream
                    let _ = cmd.execute(&self.shared_store, &mut self.connection).await;
                }
                Err(err) => {
                    let err = RESPType::Error(err.to_string());
                    let _ = self.connection.write_frame(&err).await;
                }
            };
        }
    }
}

pub async fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the listener state
    let mut server = Listener {
        listener,
        shared_store: SharedStore::new(),
    };

    server.run().await?;

    Ok(())
}
