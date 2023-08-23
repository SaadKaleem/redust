use crate::cmd::ParseError;
use crate::protocol_handler::BulkStringData;
use crate::{ConnectionBase, DataType, RESPType, SharedStore};

/// The classic GET operation in Redis
#[derive(Debug)]
pub struct Get {
    // The key to search for
    key: String,
}

impl Get {
    /// Create a new `Get` command
    pub fn new(key: String) -> Get {
        Get { key }
    }

    /// Parsing the necessary arguments for the `Get` command
    ///
    /// Syntax:
    /// GET key
    pub fn parse(cmd_strings: Vec<String>) -> Result<Get, ParseError> {
        if cmd_strings.len() != 2 {
            return Err(ParseError::SyntaxError(
                "wrong number of arguments for 'get' command".to_string(),
            ));
        } else {
            let key: String = cmd_strings[1].clone();

            return Ok(Get::new(key));
        }
    }

    /// Execute the `Get` command
    ///
    /// Get the value of key. If the key does not exist then `None` is returned
    pub async fn execute(
        self,
        shared_store: &SharedStore,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the key in the shared store
        let result: Option<DataType> = shared_store.get(self.key);

        let response: RESPType = match result {
            // Return a bulk string if there is a result of type String
            Some(val) => match val {
                DataType::String(s) => {
                    let text = s.clone();
                    let prefix_length = text.len();

                    RESPType::BulkString(Some(BulkStringData {
                        text,
                        prefix_length,
                    }))
                }
                _ => RESPType::BulkString(None),
            },
            // Return a nil bulk, if key isn't found
            None => RESPType::BulkString(None),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
