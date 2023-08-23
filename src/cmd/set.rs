use chrono::Duration;

use crate::cmd::ParseError;
use crate::protocol_handler::BulkStringData;
use crate::{ConnectionBase, DataType, RESPType, SharedStore};

/// The classic SET operation in Redis
#[derive(Debug)]
pub struct Set {
    // The key to store at
    key: String,

    // The value to be stored
    value: DataType,

    // The key expiry duration, based on EX, PX, EXAT or PXAT
    duration: Option<Duration>,

    // The NX flag (Set the key if it does not exist already)
    nx: bool,

    // The XX flag (Set the key if it already exists)
    xx: bool,

    // The GET flag to return the old string stored at key, if it existed
    get: bool,
}

impl Set {
    /// Create a new `Set` command
    pub fn new(
        key: String,
        value: DataType,
        duration: Option<Duration>,
        nx: bool,
        xx: bool,
        get: bool,
    ) -> Set {
        Set {
            key,
            value,
            duration,
            nx,
            xx,
            get,
        }
    }

    /// Parsing the necessary arguments for the `Set` command
    ///
    /// Syntax:
    /// SET key value [NX | XX] [GET]
    pub fn parse(cmd_strings: Vec<String>) -> Result<Set, ParseError> {
        if cmd_strings.len() < 3 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'set' command".to_string(),
            ));
        } else {
            let key: String = cmd_strings[1].clone();
            let value: DataType = DataType::String(cmd_strings[2].clone());

            let mut nx_flag: bool = false;
            let mut xx_flag: bool = false;
            let mut get_flag: bool = false;

            for cmd in cmd_strings.iter().skip(3) {
                match cmd.as_str() {
                    "NX" | "XX" => match Self::parse_nx_or_xx(cmd, &mut nx_flag, &mut xx_flag) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    },
                    "GET" => {
                        if get_flag == true {
                            return Err(ParseError::SyntaxError("syntax error".to_string()));
                        } else {
                            get_flag = true;
                        }
                    }
                    "EX" | "PX" | "EXAT" | "PXAT" => {
                        todo!()
                    }
                    _ => {}
                }
            }

            Ok(Set::new(key, value, None, nx_flag, xx_flag, get_flag))
        }
    }

    /// Execute the `Set` command
    ///
    /// if 'get' flag is provided, returns the old string value or `None`
    /// if the key did not exist
    pub async fn execute(
        self,
        shared_store: &SharedStore,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the key:value in the shared store
        let result: Result<Option<DataType>, ParseError> =
            shared_store.set(self.key, self.value, self.duration, self.nx, self.xx);

        let response: RESPType = match result {
            // Success: old_value for this `key` existed
            Ok(Some(value)) => match value {
                DataType::String(s) => {
                    let text = "\"".to_string() + &s.clone() + &"\"".to_string();
                    let prefix_length = text.len();

                    if self.get == true {
                        RESPType::BulkString(Some(BulkStringData {
                            text,
                            prefix_length,
                        }))
                    } else {
                        RESPType::SimpleString("\"OK\"".to_string())
                    }
                }
                _ => RESPType::SimpleString("\"OK\"".to_string()),
            },
            // Success: old_value for this `key` didn't exist
            Ok(None) => RESPType::SimpleString("\"OK\"".to_string()),
            // Failure: Possibly due to syntax error or a condition not being met
            Err(err) => RESPType::Error(err.to_string()),
        };

        cnxn.write_frame(&response).await?;

        Ok(())
    }

    fn parse_nx_or_xx(
        cmd: &String,
        nx_flag: &mut bool,
        xx_flag: &mut bool,
    ) -> Result<(), ParseError> {
        if *nx_flag == true || *xx_flag == true {
            // NX or XX were already
            return Err(ParseError::SyntaxError("NX/XX syntax error".to_string()));
        } else {
            if cmd == "NX" {
                *nx_flag = true;
            } else if cmd == "XX" {
                *xx_flag = true;
            }
            return Ok(());
        }
    }
}
