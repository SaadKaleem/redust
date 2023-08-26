// The handles the REdis Serialization Protocol parsing for all necessary types.
use std::fmt;

const MSG_SEPERATOR: &[u8; 2] = b"\r\n";
const MSG_SEPERATOR_SIZE: usize = MSG_SEPERATOR.len();

#[derive(Debug, PartialEq)]
pub enum RESPType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<BulkStringData>),
    Array(Vec<RESPType>),
}

#[derive(Debug, PartialEq)]
pub struct BulkStringData {
    pub text: String,
    pub prefix_length: usize,
}

impl fmt::Display for RESPType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RESPType::SimpleString(response) => response.fmt(fmt),
            RESPType::Error(msg) => msg.fmt(fmt),
            RESPType::Integer(num) => num.fmt(fmt),
            RESPType::BulkString(msg) => write!(fmt, "{:?}", msg),
            RESPType::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        // use space as the array element display separator
                        write!(fmt, " ")?;
                    }

                    part.fmt(fmt)?;
                }

                Ok(())
            }
        }
    }
}

trait BaseSerializer {
    const SYMBOL: &'static str;

    /// Default implementation for SimpleString, Error and Integer
    fn serialize(&self, frame: &RESPType) -> Option<Vec<u8>> {
        let serialized = format!("{}{}\r\n", self.get_symbol(), frame);
        Some(serialized.into_bytes())
    }

    fn deserialize(&self, buffer: &[u8]) -> (Option<RESPType>, usize);

    fn get_symbol(&self) -> &str;

    /// To extract the length prefix for Bulk Strings and Arrays
    fn extract_length(&self, buffer: &[u8]) -> Option<(i32, usize)> {
        // Find the position of the first occurence of '\r\n' delimiter
        let separator: Option<usize> = buffer
            .windows(2)
            .position(|window: &[u8]| window == b"\r\n");

        if let Some(crlf_pos) = separator {
            // Convert the byte slice to a string slice, skipping the initial symbol
            if let Ok(length_str) = std::str::from_utf8(&buffer[1..crlf_pos]) {
                // Parse the length_str
                if let Ok(length) = length_str.parse::<i32>() {
                    return Some((length, crlf_pos));
                }
            }
        }
        None
    }
}

struct SimpleStringSerializer;

impl BaseSerializer for SimpleStringSerializer {
    const SYMBOL: &'static str = "+";

    fn get_symbol(&self) -> &str {
        &Self::SYMBOL
    }

    fn deserialize(&self, buffer: &[u8]) -> (Option<RESPType>, usize) {
        let separator: Option<usize> = buffer
            .windows(2)
            .position(|window: &[u8]| window == b"\r\n");

        if let Some(position) = separator {
            // The separator was found
            let payload: String = String::from_utf8_lossy(&buffer[1..position]).to_string();
            return (Some(RESPType::SimpleString(payload)), position + 2);
        } else {
            // The separator was not found
            return (None, 0);
        }
    }
}

struct ErrorSerializer;

impl BaseSerializer for ErrorSerializer {
    const SYMBOL: &'static str = "-";

    fn get_symbol(&self) -> &str {
        &Self::SYMBOL
    }

    fn deserialize(&self, buffer: &[u8]) -> (Option<RESPType>, usize) {
        let separator: Option<usize> = buffer
            .windows(2)
            .position(|window: &[u8]| window == b"\r\n");

        if let Some(position) = separator {
            // The separator was found
            let payload: String = String::from_utf8_lossy(&buffer[1..position]).to_string();
            return (Some(RESPType::Error(payload)), position + 2);
        } else {
            // The separator was not found
            return (None, 0);
        }
    }
}

struct IntegerSerializer;

impl BaseSerializer for IntegerSerializer {
    const SYMBOL: &'static str = ":";

    fn get_symbol(&self) -> &str {
        &Self::SYMBOL
    }

    fn deserialize(&self, buffer: &[u8]) -> (Option<RESPType>, usize) {
        let separator: Option<usize> = buffer
            .windows(2)
            .position(|window: &[u8]| window == b"\r\n");

        if let Some(position) = separator {
            // The separator was found
            let payload: Result<i64, _> = String::from_utf8_lossy(&buffer[1..position]).parse();

            match payload {
                Ok(integer) => {
                    return (Some(RESPType::Integer(integer)), position + 2);
                }
                Err(_) => {
                    println!("Failed to parse integer.");
                    return (None, 0);
                }
            }
        } else {
            // The separator was not found
            return (None, 0);
        }
    }
}

struct BulkStringSerializer;

impl BaseSerializer for BulkStringSerializer {
    const SYMBOL: &'static str = "$";

    fn get_symbol(&self) -> &str {
        &Self::SYMBOL
    }

    fn serialize(&self, frame: &RESPType) -> Option<Vec<u8>> {
        match frame {
            RESPType::BulkString(values) => match values {
                Some(data) => {
                    return Some(
                        format!(
                            "{}{}\r\n{}\r\n",
                            self.get_symbol(),
                            data.prefix_length,
                            data.text
                        )
                        .into_bytes(),
                    )
                }
                None => return Some(format!("{}{}\r\n", self.get_symbol(), -1).into_bytes()),
            },
            _ => return None,
        };
    }

    fn deserialize(&self, buffer: &[u8]) -> (Option<RESPType>, usize) {
        // Found a prefix length, with the start_pos of the 1st CRLF
        let length_and_pos = self.extract_length(buffer);

        if let Some((prefix_length, crlf_pos)) = length_and_pos {
            if prefix_length == -1 {
                return (Some(RESPType::BulkString(None)), 5);
            } else {
                let prefix_length_u = prefix_length as usize;

                // Iterate from `crlf_pos + 2` for `prefix_length` iterations, and then
                // ensure CRLF exists.
                let start_index: usize = crlf_pos + 2;
                let end_index: usize = start_index + prefix_length_u;

                // Create a new buffer containing the specified bytes
                let new_buffer: Vec<u8>;
                if prefix_length == 0 {
                    new_buffer = "".as_bytes().to_vec();
                } else {
                    new_buffer = buffer[start_index..end_index].to_vec();
                }

                if new_buffer.len() != prefix_length_u {
                    return (None, 0);
                } else {
                    // Ensure end_index is within bounds
                    if (end_index + 2) > buffer.len() {
                        return (None, 0);
                    }

                    // Verify the two bytes after the end index are CRLF
                    if buffer[end_index..end_index + 2] != *MSG_SEPERATOR {
                        return (None, 0);
                    } else {
                        // Unwrap to string, and construct the enum
                        let text = String::from_utf8(new_buffer).unwrap();
                        let bulk_str_data = BulkStringData {
                            text,
                            prefix_length: prefix_length_u,
                        };

                        return (
                            Some(RESPType::BulkString(Some(bulk_str_data))),
                            1 + prefix_length_u.to_string().len()
                                + MSG_SEPERATOR_SIZE
                                + prefix_length_u
                                + MSG_SEPERATOR_SIZE,
                        );
                    }
                }
            }
        } else {
            // Did not find prefix_length and CRLF
            return (None, 0);
        }
    }
}

struct ArraySerializer;

impl BaseSerializer for ArraySerializer {
    const SYMBOL: &'static str = "*";

    fn get_symbol(&self) -> &str {
        &Self::SYMBOL
    }

    fn serialize(&self, frame: &RESPType) -> Option<Vec<u8>> {
        match frame {
            RESPType::Array(vec) => {
                // Define a mutable buffer to return
                let mut buffer: Vec<u8> = self.get_symbol().as_bytes().to_vec();

                // Convert the vector length to a string, and then to ascii respresentation
                let vec_length = vec.len().to_string();
                let arr_length_ascii = vec_length.as_bytes();
                buffer.extend(arr_length_ascii);

                // Add the seperator
                buffer.extend(MSG_SEPERATOR);

                // Serialize the data for children items, within the Array.
                for item in vec {
                    let result = serialize_data(item);
                    match result {
                        Some(child_payload) => buffer.extend(child_payload),
                        None => return None,
                    }
                }

                return Some(buffer);
            }
            _ => return None,
        };
    }

    fn deserialize(&self, buffer: &[u8]) -> (Option<RESPType>, usize) {
        // Found a prefix length, with the start_pos of the 1st CRLF
        let length_and_pos = self.extract_length(buffer);

        if let Some((array_length, crlf_pos)) = length_and_pos {
            if array_length == 0 {
                return (Some(RESPType::Array(Vec::new())), 2 + MSG_SEPERATOR_SIZE);
            } else {
                // Iterate from `crlf_pos + 2` for `array_length` iterations
                let mut next_index: usize = crlf_pos + 2;

                let mut vec: Vec<RESPType> = Vec::new();

                for _ in 0..array_length {
                    let (result, length) = deserialize_buffer(&buffer[next_index..]);
                    match result {
                        Some(child_payload) => vec.push(child_payload),
                        None => return (None, 0),
                    }
                    next_index = next_index + length;
                }
                (Some(RESPType::Array(vec)), next_index)
            }
        } else {
            // Did not find prefix_length and CRLF
            return (None, 0);
        }
    }
}

pub fn deserialize_buffer(input_buf: &[u8]) -> (Option<RESPType>, usize) {
    if let Some(&first_byte) = input_buf.first() {
        match first_byte {
            b'+' => SimpleStringSerializer.deserialize(&input_buf),
            b'-' => ErrorSerializer.deserialize(&input_buf),
            b':' => IntegerSerializer.deserialize(&input_buf),
            b'$' => BulkStringSerializer.deserialize(&input_buf),
            b'*' => ArraySerializer.deserialize(&input_buf),
            _ => (None, 0),
        }
    } else {
        // Handle the case when the buffer is empty
        (None, 0)
    }
}

pub fn serialize_data(input_data: &RESPType) -> Option<Vec<u8>> {
    match input_data {
        RESPType::SimpleString(_) => SimpleStringSerializer.serialize(&input_data),
        RESPType::Error(_) => ErrorSerializer.serialize(&input_data),
        RESPType::Integer(_) => IntegerSerializer.serialize(&input_data),
        RESPType::BulkString(_) => BulkStringSerializer.serialize(&input_data),
        RESPType::Array(_) => ArraySerializer.serialize(&input_data),
    }
}
