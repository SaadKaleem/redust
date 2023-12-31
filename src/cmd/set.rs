use chrono::{Duration, Utc};

use crate::cmd::ParseError;
use crate::protocol_handler::BulkStringData;
use crate::{ConnectionBase, DataType, RESPType, SharedStoreBase};

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
    /// SET key value [NX | XX] [GET] [EX seconds | PX milliseconds |
    ///    EXAT unix-time-seconds | PXAT unix-time-milliseconds]
    ///
    pub fn parse(cmd_strings: Vec<String>) -> Result<Set, ParseError> {
        if cmd_strings.len() < 3 {
            return Err(ParseError::SyntaxError(
                "ERR wrong number of arguments for 'set' command".to_string(),
            ));
        } else {
            let key: String = cmd_strings[1].clone();
            let value: DataType = DataType::String(cmd_strings[2].clone());
            let mut duration: Option<Duration> = None;

            let mut nx_flag: bool = false;
            let mut xx_flag: bool = false;
            let mut get_flag: bool = false;

            let mut iterator = cmd_strings.iter().skip(3);

            while let Some(cmd_arg) = iterator.next() {
                match cmd_arg.as_str() {
                    "NX" | "XX" => {
                        match Self::parse_nx_or_xx(cmd_arg, &mut nx_flag, &mut xx_flag) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    "GET" => {
                        if get_flag == true {
                            return Err(ParseError::SyntaxError("syntax error".to_string()));
                        } else {
                            get_flag = true;
                        }
                    }
                    "EX" | "PX" | "EXAT" | "PXAT" => {
                        // Get the next iterator value, and also check if it parses properly.
                        match iterator.next() {
                            None => {
                                return Err(ParseError::SyntaxError("syntax error".to_string()))
                            }
                            Some(next_arg) => {
                                match Self::parse_duration_args(cmd_arg, next_arg, &mut duration) {
                                    Ok(_) => {}
                                    Err(err) => return Err(err),
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Ok(Set::new(key, value, duration, nx_flag, xx_flag, get_flag))
        }
    }

    /// Execute the `Set` command
    ///
    /// if 'get' flag is provided, returns the old string value or `None`
    /// if the key did not exist
    pub async fn execute(
        self,
        shared_store: &dyn SharedStoreBase,
        cnxn: &mut dyn ConnectionBase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the key:value in the shared store
        let result: Result<Option<DataType>, ParseError> =
            shared_store.set(self.key, self.value, self.duration, self.nx, self.xx);

        let response: RESPType = match result {
            // Success: old_value for this `key` existed
            Ok(Some(value)) => match value {
                DataType::String(s) => {
                    if self.get == true {
                        let text = "\"".to_string() + &s.clone() + &"\"".to_string();
                        let prefix_length = text.len();

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

    /// Helper method to parse the duration arguments
    ///
    /// `cmd_arg` would be the EX | PX | EXAT | PXAT in our case.
    /// `next_arg` must be a parsable integer.
    fn parse_duration_args(
        cmd_arg: &String,
        next_arg: &String,
        duration: &mut Option<Duration>,
    ) -> Result<(), ParseError> {
        match duration {
            // If duration is already set, we obviously need to throw a syntax error.
            Some(_) => Err(ParseError::SyntaxError(format!("{} syntax error", cmd_arg))),
            None => {
                // Parse the next argument, to see if it fulfils an i64.
                let time_value = next_arg.parse::<i64>();

                match time_value {
                    Err(_) => {
                        return Err(ParseError::SyntaxError(format!("{} syntax error", cmd_arg)));
                    }
                    Ok(time_value) => {
                        match cmd_arg.as_str() {
                            "EX" => *duration = Some(Duration::seconds(time_value)),
                            "PX" => *duration = Some(Duration::milliseconds(time_value)),
                            "EXAT" => {
                                // Get the diff from now, and set the duration
                                let diff_seconds = time_value - Utc::now().timestamp();
                                *duration = Some(Duration::seconds(diff_seconds))
                            }
                            "PXAT" => {
                                // Convert to seconds, and then assign the duration
                                let diff_seconds = (time_value / 1000) - Utc::now().timestamp();
                                *duration = Some(Duration::seconds(diff_seconds))
                            }
                            _ => {}
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
}
