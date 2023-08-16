use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{deserialize_buffer, serialize_data, RESPType};

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

    pub async fn read_frame(&mut self) -> Result<Option<RESPType>, Box<dyn std::error::Error>> {
        loop {
            // Attempt to deserialize a frame from the data in the buffer.
            // If successful, a `RESPType` frame is returned.
            //
            // Since an array is length-prefixed, and there can
            // If a partial frame is in the buffer, this will return `None` and `0`
            // Therefore, we will keep reading more data.
            //
            // If the frame does not fit into the buffer, we will reallocate space anyway, to keep reading.
            println!("Buffer before reading: {:?}", self.buffer);
            let (frame, _) = deserialize_buffer(&self.buffer.as_slice());
            // If we got a valid frame, return it
            if frame.is_some() {
                return Ok(frame);
            }

            println!("Buffer after reading: {:?}", self.buffer);

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
    pub async fn write_frame(&mut self, frame: &RESPType) -> io::Result<()> {
        let data = serialize_data(&frame).unwrap();

        self.stream.write_all(&data).await?;

        // Make sure that any buffered contents are written.
        self.stream.flush().await
    }
}
