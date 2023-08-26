use crate::cmd::ParseError;
use crate::{ConnectionBase, RESPType, SharedStoreBase};

/// The LPUSH operation in Redis
#[derive(Debug)]
pub struct Lpush {
    // The keys to push at
    key: String,

    // The elements to push
    elements: Vec<String>,
}

impl Lpush {
    /// Create a new `LPUSH` command
    pub fn new(key: String, elements: Vec<String>) -> Lpush {
        Lpush { key, elements }
    }

    /// Parsing the necessary arguments for the `LPUSH` command
    ///
    /// Syntax:
    /// LPUSH key
    pub fn parse(cmd_strings: Vec<String>) -> Result<Lpush, ParseError> {
        if cmd_strings.len() < 3 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'lpush' command".to_string(),
            ));
        } else {
            Ok(Lpush::new(
                cmd_strings[1].clone(),
                cmd_strings[2..].to_vec(),
            ))
        }
    }

    /// Execute the `Lpush` command
    ///
    /// Returns an integer reply, for the number of keys
    /// that exist from the specified arguments
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the key:value in the shared store
        let result = shared_store.lpush(self.key, self.elements);

        let response = match result {
            Ok(val) => RESPType::Integer(val),
            Err(err) => RESPType::Error(err.to_string()),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
