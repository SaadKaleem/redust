use crate::cmd::ParseError;
use crate::{Connection, RESPType};

#[derive(Debug, Default)]
pub struct Echo {
    msg: String,
}

impl Echo {
    /// Create a new `Echo` command with a `msg`
    pub fn new(msg: String) -> Echo {
        Echo { msg }
    }

    pub fn parse(cmd_strings: Vec<String>) -> Result<Echo, ParseError> {
        if cmd_strings.len() > 2 {
            return Err(ParseError::ExtraCmdArg(
                "ERR wrong number of arguments for 'echo' command".to_string(),
            ));
        } else {
            match cmd_strings.get(1) {
                Some(msg) => return Ok(Echo::new(msg.into())),
                None => {
                    return Err(ParseError::MissingCmdArg(
                        "ERR wrong number of arguments for 'echo' command".to_string(),
                    ))
                }
            }
        }
    }

    /// Execute the `Echo` command
    pub async fn execute(self, cnxn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
        let resp = RESPType::SimpleString(format!("{}{}{}", "\"", self.msg, "\""));

        // Write the response back to the client
        cnxn.write_frame(&resp).await?;

        Ok(())
    }
}
