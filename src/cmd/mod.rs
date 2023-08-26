mod ping;
pub use ping::Ping;

mod echo;
pub use echo::Echo;

mod set;
pub use set::Set;

mod get;
pub use get::Get;

mod exists;
pub use exists::Exists;

mod del;
pub use del::Del;

mod incr;
pub use incr::Incr;

mod decr;
pub use decr::Decr;

mod lpush;
pub use lpush::Lpush;

mod lrange;
pub use lrange::Lrange;

use crate::{Connection, RESPType, SharedStore};
use std::fmt;

/// Methods called on `Command` are delegated to the command implementation.
/// `Command` essentially is acting as a Catalog
#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Exists(Exists),
    Del(Del),
    Incr(Incr),
    Decr(Decr),
    Lpush(Lpush),
    Lrange(Lrange),
}

#[derive(Debug)]
pub enum ParseError {
    SyntaxError(String),
    ConditionNotMet(String),
    UnrecognizedCmd(String),
    ExpectedArrayType(String),
    ExpectedStringType(String),
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::SyntaxError(msg) => msg.fmt(fmt),
            ParseError::ConditionNotMet(msg) => msg.fmt(fmt),
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
                    "ERR Expected Array got {:?}",
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

        let cmd_name = cmd_strings.get(0).unwrap();

        // println!("{:?}", cmd_name.as_str());
        let cmd: Command = match cmd_name.to_lowercase().as_str() {
            "ping" => Command::Ping(Ping::parse(cmd_strings)?),
            "echo" => Command::Echo(Echo::parse(cmd_strings)?),
            "set" => Command::Set(Set::parse(cmd_strings)?),
            "get" => Command::Get(Get::parse(cmd_strings)?),
            "exists" => Command::Exists(Exists::parse(cmd_strings)?),
            "del" => Command::Del(Del::parse(cmd_strings)?),
            "incr" => Command::Incr(Incr::parse(cmd_strings)?),
            "decr" => Command::Decr(Decr::parse(cmd_strings)?),
            "lpush" => Command::Lpush(Lpush::parse(cmd_strings)?),
            "lrange" => Command::Lrange(Lrange::parse(cmd_strings)?),
            _ => {
                return Err(ParseError::UnrecognizedCmd(format!(
                    "unknown command '{}'",
                    cmd_name
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
                    "ERR Expected String Type got {:?}",
                    frame
                )))
            }
        };

        Ok(text)
    }

    pub async fn execute(
        self,
        shared_store: &SharedStore,
        cnxn: &mut Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Ping(cmd) => cmd.execute(cnxn).await,
            Command::Echo(cmd) => cmd.execute(cnxn).await,
            Command::Set(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Get(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Exists(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Del(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Incr(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Decr(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Lpush(cmd) => cmd.execute(shared_store, cnxn).await,
            Command::Lrange(cmd) => cmd.execute(shared_store, cnxn).await,
        }
    }
}
