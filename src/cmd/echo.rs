use crate::cmd::ParseError;
use crate::{ConnectionBase, RESPType};

#[derive(Debug, Default)]
pub struct Echo {
    msg: String,
}

impl Echo {
    /// Create a new `Echo` command with a `msg`
    pub fn new(msg: String) -> Echo {
        Echo { msg }
    }

    /// Parsing the necessary arguments for the `Echo` command
    ///
    /// Syntax:
    /// ECHO message
    pub fn parse(cmd_strings: Vec<String>) -> Result<Echo, ParseError> {
        if cmd_strings.len() > 2 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'echo' command".to_string(),
            ));
        } else {
            match cmd_strings.get(1) {
                Some(msg) => return Ok(Echo::new(msg.into())),
                None => {
                    return Err(ParseError::SyntaxError(
                        "ERR wrong number of arguments for 'echo' command".to_string(),
                    ))
                }
            }
        }
    }

    /// Execute the `Echo` command
    pub async fn execute(
        self,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resp = RESPType::SimpleString(format!("{}{}{}", "\"", self.msg, "\""));

        // Write the response back to the client
        let result = cnxn.write_frame(&resp).await;

        match result {
            Err(err) => Err(Box::new(err)), // Propagate the error
            _ => Ok(()),                    // No Error, return Ok
        }
    }
}
