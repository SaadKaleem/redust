use crate::cmd::ParseError;
use crate::{Connection, RESPType};

#[derive(Debug, Default)]
pub struct Ping {}

impl Ping {
    /// Create a new `Ping` command
    pub fn new() -> Ping {
        Ping {}
    }

    pub fn parse(cmd_strings: Vec<String>) -> Result<Ping, ParseError> {
        if cmd_strings.len() > 1 {
            Err(ParseError::ExtraCmdArg(
                "ERR: Found more than one arg for PING".to_string(),
            ))
        } else {
            Ok(Ping::new())
        }
    }

    /// Execute the "Ping" command and return PONG
    pub async fn execute(self, cnxn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
        let resp = RESPType::SimpleString("PONG".to_string());

        // Write the response back to the client
        let _ = cnxn.write_frame(&resp).await;

        Ok(())
    }
}
