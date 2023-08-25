use crate::cmd::ParseError;
use crate::{ConnectionBase, RESPType, SharedStoreBase};

/// The INCR operation in Redis
#[derive(Debug)]
pub struct Incr {
    // The keys to check if they exist
    key: String,
}

impl Incr {
    /// Create a new `INCR` command
    pub fn new(key: String) -> Incr {
        Incr { key }
    }

    /// Parsing the necessary arguments for the `INCR` command
    ///
    /// Syntax:
    /// INCR key
    pub fn parse(cmd_strings: Vec<String>) -> Result<Incr, ParseError> {
        if cmd_strings.len() != 2 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'incr' command".to_string(),
            ));
        } else {
            Ok(Incr::new(cmd_strings[1].clone()))
        }
    }

    /// Execute the `Incr` command
    ///
    /// Returns an integer reply, for the number of keys
    /// that exist from the specified arguments
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the key:value in the shared store
        let result = shared_store.incr(self.key);

        let response = match result {
            Ok(val) => RESPType::Integer(val),
            Err(err) => RESPType::Error(err.to_string()),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
