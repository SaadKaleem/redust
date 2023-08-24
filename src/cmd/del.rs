use crate::cmd::ParseError;
use crate::{ConnectionBase, RESPType, SharedStoreBase};

/// The DEL operation in Redis
#[derive(Debug)]
pub struct Del {
    // The keys to check if they exist
    keys: Vec<String>,
}

impl Del {
    /// Create a new `DEL` command
    pub fn new(keys: Vec<String>) -> Del {
        Del { keys }
    }

    /// Parsing the necessary arguments for the `DEL` command
    ///
    /// Syntax:
    /// DEL key [key ...]
    pub fn parse(cmd_strings: Vec<String>) -> Result<Del, ParseError> {
        if cmd_strings.len() < 2 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'del' command".to_string(),
            ));
        } else {
            let keys = cmd_strings[1..].to_vec();

            Ok(Del::new(keys))
        }
    }

    /// Execute the `Del` command
    ///
    /// Returns an integer reply, for the number of keys
    /// that exist from the specified arguments
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the key:value in the shared store
        let result: u64 = shared_store.del(self.keys);

        let response = RESPType::Integer(result as i64);

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
