use crate::cmd::ParseError;
use crate::{ConnectionBase, RESPType, SharedStoreBase};

/// The DECR operation in Redis
#[derive(Debug)]
pub struct Decr {
    // The keys to check if they exist
    key: String,
}

impl Decr {
    /// Create a new `DECR` command
    pub fn new(key: String) -> Decr {
        Decr { key }
    }

    /// Parsing the necessary arguments for the `DECR` command
    ///
    /// Syntax:
    /// DECR key
    pub fn parse(cmd_strings: Vec<String>) -> Result<Decr, ParseError> {
        if cmd_strings.len() != 2 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'decr' command".to_string(),
            ));
        } else {
            Ok(Decr::new(cmd_strings[1].clone()))
        }
    }

    /// Execute the `Decr` command
    ///
    /// Returns an integer reply, for the number of keys
    /// that exist from the specified arguments
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the key:value in the shared store
        let result = shared_store.decr(self.key);

        let response = match result {
            Ok(val) => RESPType::Integer(val),
            Err(err) => RESPType::Error(err.to_string()),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
