use crate::cmd::ParseError;
use crate::protocol_handler::BulkStringData;
use crate::{ConnectionBase, RESPType, SharedStoreBase};

/// The LRANGE operation in Redis
#[derive(Debug)]
pub struct Lrange {
    // The key to query, which represents a List
    key: String,

    // The start index
    start: i64,

    // The stop index (inclusive)
    stop: i64,
}

impl Lrange {
    /// Create a new `LRANGE` command
    pub fn new(key: String, start: i64, stop: i64) -> Lrange {
        Lrange { key, start, stop }
    }

    /// Parsing the necessary arguments for the `LRANGE` command
    ///
    /// Syntax:
    /// LRANGE key
    pub fn parse(cmd_strings: Vec<String>) -> Result<Lrange, ParseError> {
        if cmd_strings.len() != 4 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'lrange' command".to_string(),
            ));
        } else {
            let start: i64;
            let stop: i64;

            match cmd_strings[2].parse::<i64>() {
                Ok(val) => {
                    start = val;
                }
                Err(_) => {
                    return Err(ParseError::SyntaxError(
                        "ERR start index is not an integer".to_string(),
                    ));
                }
            }

            match cmd_strings[3].parse::<i64>() {
                Ok(val) => {
                    stop = val;
                }
                Err(_) => {
                    return Err(ParseError::SyntaxError(
                        "ERR stop index is not an integer".to_string(),
                    ));
                }
            }

            Ok(Lrange::new(cmd_strings[1].clone(), start, stop))
        }
    }

    /// Execute the `Lrange` command
    ///
    /// Returns the elements, as an Array which are part of the
    /// provided indices
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Query the `key` for the elements
        let result = shared_store.lrange(self.key, self.start, self.stop);

        let response = match result {
            Ok(string_vec) => {
                // Convert Vec<String> to Vec<RESPType of Bulk Strings>
                let bulk_strings: Vec<RESPType> = string_vec
                    .iter()
                    .map(|s| {
                        RESPType::BulkString(Some(BulkStringData {
                            text: s.to_string(),
                            prefix_length: s.len() as usize,
                        }))
                    })
                    .collect();

                RESPType::Array(bulk_strings)
            }
            Err(err) => RESPType::Error(err.to_string()),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
