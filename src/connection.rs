use crate::{deserialize_buffer, serialize_data, RESPType};
use async_trait::async_trait;
use mockall::automock;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Cannot have `mockall` as a dev-dependency
/// and also import the Mocked attr in the tests/ dir
///
/// "those in a tests directory, behave like independent crates that use your main library.
/// As a consequence the library itself is not compiled in test mode for integration tests,
/// and your cfg_attr disables automock"
///
/// Refer to:
/// https://stackoverflow.com/q/76831451
/// https://github.com/rust-lang/cargo/issues/2911
///
#[automock]
#[async_trait]
pub trait ConnectionBase: Send + Sync {
    async fn read_frame(&mut self) -> Result<Option<RESPType>, Box<dyn std::error::Error>>;

    async fn write_frame(&mut self, frame: &RESPType) -> tokio::io::Result<()>;
}

/// The purpose of `Connection` is to read and write frames on the
/// underlying `TcpStream`, which is established between the client
/// and the server.
///
/// When reading frames, the `Connection` uses an internal buffer,
/// of type Vec<u8> which is filled up until we get a valid frame
/// the `Connection` calls the protocol handler to deserialize the
/// bytes, and if a valid frame is found, it is returned to the caller.
///
/// When writing frames, the data is first serialized into the RESP format
/// and then all of it is written into the TCP Stream
#[derive(Debug)]
pub struct Connection {
    // The TCP Stream for reading and writing to the client
    stream: TcpStream,

    // The buffer for reading frames.
    buffer: Vec<u8>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: socket,
            // Default to a 4KB read buffer, this should be configured
            // based on the requirements of the application, as a greater
            // buffer might improve performance
            // It is allowed to reallocate to increase its capacity
            //
            // Panics: if capacity exceeds `isize::MAX`
            // TODO: This needs to change
            buffer: Vec::with_capacity(4 * 1024),
        }
    }
}

#[async_trait]
impl ConnectionBase for Connection {
    async fn read_frame(&mut self) -> Result<Option<RESPType>, Box<dyn std::error::Error>> {
        loop {
            // Attempt to deserialize a frame from the data in the buffer.
            // If successful, a `RESPType` frame is returned.
            //
            // Since an array is length-prefixed, and there can
            // If a partial frame is in the buffer, this will return `None` and `0`
            // Therefore, we will keep reading more data.
            //
            // If the frame does not fit into the buffer, we will reallocate space anyway, to keep reading.
            let (frame, frame_size) = deserialize_buffer(&self.buffer.as_slice());
            // If we got a valid frame, return it and
            // drain the buffer upto the frame_size
            if frame.is_some() {
                self.buffer.drain(0..frame_size);
                return Ok(frame);
            }

            // Attempt to read more data from the socket
            //
            // On success, the number of bytes read are written into `buffer`
            // `0` indicates the end of the stream
            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                // Remote peer closed the connection, check if buffer has any existing data
                // Since this would not be a clean shutdown
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    // Remote peer closed socket while sending data
                    return Err("Connection Reset by Peer".into());
                }
            }
        }
    }

    /// Serializes the frame and attempt to
    /// write the whole buffer to the TCPStream
    async fn write_frame(&mut self, frame: &RESPType) -> io::Result<()> {
        let data = serialize_data(&frame).unwrap();

        // println!("{:?}", String::from_utf8(data.clone()));
        self.stream.write_all(&data).await?;

        // Make sure that any buffered contents are written.
        self.stream.flush().await
    }
}
