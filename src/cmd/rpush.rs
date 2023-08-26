use crate::cmd::ParseError;
use crate::{ConnectionBase, RESPType, SharedStoreBase};

/// The RPUSH operation in Redis
#[derive(Debug)]
pub struct Rpush {
    // The keys to push at
    key: String,

    // The elements to push
    elements: Vec<String>,
}

impl Rpush {
    /// Create a new `RPUSH` command
    pub fn new(key: String, elements: Vec<String>) -> Rpush {
        Rpush { key, elements }
    }

    /// Parsing the necessary arguments for the `RPUSH` command
    ///
    /// Syntax:
    /// RPUSH key element [element ...]
    pub fn parse(cmd_strings: Vec<String>) -> Result<Rpush, ParseError> {
        if cmd_strings.len() < 3 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'rpush' command".to_string(),
            ));
        } else {
            Ok(Rpush::new(
                cmd_strings[1].clone(),
                cmd_strings[2..].to_vec(),
            ))
        }
    }

    /// Execute the `Rpush` command
    ///
    /// Returns an integer reply, representing
    /// the length of the list
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Push the elements in the shared store
        let result = shared_store.rpush(self.key, self.elements);

        let response = match result {
            Ok(val) => RESPType::Integer(val),
            Err(err) => RESPType::Error(err.to_string()),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }
}
