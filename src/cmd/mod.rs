mod ping;
pub use ping::Ping;

mod echo;
pub use echo::Echo;

use crate::{Connection, RESPType};
use std::fmt;

/// Methods called on `Command` are delegated to the command implementation.
/// `Command` essentially is acting as a Catalog
#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
}

#[derive(Debug)]
pub enum ParseError {
    MissingCmdArg(String),
    ExtraCmdArg(String),
    UnrecognizedCmd(String),
    ExpectedArrayType(String),
    ExpectedStringType(String),
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::MissingCmdArg(msg) => msg.fmt(fmt),
            ParseError::ExtraCmdArg(msg) => msg.fmt(fmt),
            ParseError::UnrecognizedCmd(msg) => msg.fmt(fmt),
            ParseError::ExpectedArrayType(msg) => msg.fmt(fmt),
            ParseError::ExpectedStringType(msg) => msg.fmt(fmt),
        }
    }
}

impl Command {
    /// Parse a command from a deserialized RESPType, which
    /// must be an array RESPType, with String types
    pub fn infer_command(frame: RESPType) -> Result<Command, ParseError> {
        let array: Vec<RESPType> = match frame {
            RESPType::Array(array) => array,
            frame => {
                return Err(ParseError::ExpectedArrayType(format!(
                    "ERR: Expected Array got {:?}",
                    frame
                )))
            }
        };

        let mut cmd_strings: Vec<String> = Vec::new();

        // Populate all the strings
        for item in array.into_iter() {
            let text: String = Command::extract_string(item)?;
            cmd_strings.push(text);
        }

        let cmd_name = cmd_strings.get(0).unwrap().to_lowercase();

        println!("{:?}", cmd_name.as_str());
        let cmd = match cmd_name.as_str() {
            "ping" => Command::Ping(Ping::parse(cmd_strings)?),
            "echo" => Command::Echo(Echo::parse(cmd_strings)?),
            _ => {
                println!("Unrecognized Command");
                return Err(ParseError::UnrecognizedCmd(format!(
                    "ERR: Unrecognized Command"
                )));
            }
        };

        Ok(cmd)
    }

    pub fn extract_string(frame: RESPType) -> Result<String, ParseError> {
        let text: String = match frame {
            RESPType::BulkString(val) => val.unwrap().text,
            RESPType::SimpleString(val) => val,
            _ => {
                return Err(ParseError::ExpectedStringType(format!(
                    "ERR: Expected String Type got {:?}",
                    frame
                )))
            }
        };

        Ok(text)
    }

    pub async fn execute(self, cnxn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Ping(cmd) => cmd.execute(cnxn).await,
            Command::Echo(cmd) => cmd.execute(cnxn).await,
        }
    }
}