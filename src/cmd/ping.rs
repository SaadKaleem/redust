use crate::cmd::ParseError;
use crate::{Connection, RESPType};

#[derive(Debug, Default)]
pub struct Ping {
    msg: Option<String>,
}

impl Ping {
    /// Create a new `Ping` command
    pub fn new(msg: Option<String>) -> Ping {
        Ping { msg }
    }

    pub fn parse(cmd_strings: Vec<String>) -> Result<Ping, ParseError> {
        if cmd_strings.len() > 2 {
            return Err(ParseError::SyntaxError(
                "wrong number of arguments for 'ping' command".to_string(),
            ));
        } else {
            // Get the value at index 1, and pass the argument so it can be cloned in a different closure.
            let msg = cmd_strings.get(1).map(|s| s.clone());

            match msg {
                Some(val) => {
                    return Ok(Ping::new(Some(val)));
                }
                None => {
                    return Ok(Ping::new(None));
                }
            }
        }
    }

    /// Execute the "Ping" command and return PONG or the optional message if provided.
    pub async fn execute(self, cnxn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
        let resp = match self.msg {
            Some(msg) => RESPType::SimpleString(format!("{}{}{}", "\"", msg, "\"")),
            None => RESPType::SimpleString("\"PONG\"".to_string()),
        };

        // Write the response back to the client
        let _ = cnxn.write_frame(&resp).await;

        Ok(())
    }
}
