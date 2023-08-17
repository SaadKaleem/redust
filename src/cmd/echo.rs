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
        match cmd_strings.get(1) {
            Some(msg) => Ok(Echo::new(msg.into())),
            None => Err(ParseError::MissingCmdArg(
                "ERR: Missing Argument".to_string(),
            )),
        }
    }

    /// Execute the `Echo` command
    pub async fn execute(self, cnxn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
        let resp = RESPType::SimpleString(self.msg.to_string());

        // Write the response back to the client
        cnxn.write_frame(&resp).await?;

        Ok(())
    }
}
