pub mod cmd;
pub mod connection;
pub mod data_store;
pub use connection::Connection;
pub use connection::ConnectionBase;
pub use connection::MockConnectionBase;
pub use data_store::DataType;
pub use data_store::MockSharedStoreBase;
pub use data_store::SharedStore;
pub use data_store::SharedStoreBase;
pub mod protocol_handler;
pub use protocol_handler::deserialize_buffer;
pub use protocol_handler::serialize_data;
pub use protocol_handler::RESPType;
pub mod server;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 6666;
