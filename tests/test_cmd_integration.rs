use chrono::{Duration, Utc};
use redis::{Client, Connection, RedisResult};
use redust::{DEFAULT_HOST, DEFAULT_PORT};
use rstest::fixture;
use rstest::rstest;

#[fixture]
fn cnxn() -> Connection {
    let client = Client::open(format!("redis://{}:{}/", DEFAULT_HOST, DEFAULT_PORT)).unwrap();
    let cnxn = client.get_connection().unwrap();
    cnxn
}

#[rstest]
// No Arguments
#[case("PING", None, None)]
// Single Argument
#[case("PING", Some("Hello, World!".to_string()), None)]
// Extra Argument
#[case("PING", Some("Hello, World!".to_string()), Some("Hello, World!".to_string()))]
fn test_ping(
    #[case] command: &str,
    #[case] message: Option<String>,
    #[case] extra_arg: Option<String>,
    mut cnxn: Connection,
) -> RedisResult<()> {
    let response: Result<String, redis::RedisError> = redis::cmd(command)
        .arg(&message)
        .arg(&extra_arg)
        .query(&mut cnxn);

    if extra_arg.is_some() {
        assert_eq!(
            response.err().unwrap().to_string(),
            "wrong: number of arguments for 'ping' command".to_string()
        );
    } else if message.is_some() {
        assert_eq!(
            response.unwrap(),
            format!("{}{}{}", "\"", message.unwrap(), "\"")
        );
    } else {
        assert_eq!(response.unwrap(), format!("{}{}{}", "\"", "PONG", "\""));
    }

    Ok(())
}

#[rstest]
// Single Argument
#[case("ECHO", Some("Hello, World!".to_string()), None)]
// Missing Argument
#[case("ECHO", None, None)]
// Extra Argument
#[case("ECHO", Some("Hello, World!".to_string()), Some("Hello, World!".to_string()))]
fn test_echo(
    #[case] command: &str,
    #[case] message: Option<String>,
    #[case] extra_arg: Option<String>,
    mut cnxn: Connection,
) -> RedisResult<()> {
    let response: Result<String, redis::RedisError> = redis::cmd(command)
        .arg(&message)
        .arg(&extra_arg)
        .query(&mut cnxn);

    if extra_arg.is_some() || message.is_none() {
        assert_eq!(
            response.err().unwrap().to_string(),
            "An error was signalled by the server - ResponseError: wrong number of arguments for 'echo' command".to_string()
        );
    } else {
        assert_eq!(
            response.unwrap(),
            format!("{}{}{}", "\"", message.unwrap(), "\"")
        );
    }

    Ok(())
}

#[rstest]
// Missing Key Argument
#[case("SET", None, None, None, None, None, None, Err(String::from("An error was signalled by the server - ResponseError: wrong number of arguments for 'set' command")))]
// Missing Value Argument
#[case(
    "SET",
    Some(String::from("John")),
    None,
    None,
    None,
    None,
    None,
    Err(String::from("An error was signalled by the server - ResponseError: wrong number of arguments for 'set' command"))
)]
// Two Arguments (Key & Value)
#[case(
    "SET",
    Some(String::from("John")),
    Some(String::from("Doe")),
    None,
    None,
    None,
    None,
    Ok(String::from("\"OK\""))
)]
// With NX
#[case(
    "SET",
    Some(String::from("William")),
    Some(String::from("Bryce")),
    Some(String::from("NX")),
    None,
    None,
    None,
    Ok(String::from("\"OK\""))
)]
// With XX
#[case(
    "SET",
    Some(String::from("Crickett")),
    Some(String::from("Test")),
    Some(String::from("XX")),
    None,
    None,
    None,
    Err(String::from("XX: condition not met"))
)]
// With NX and GET
#[case(
    "SET",
    Some(String::from("Hello")),
    Some(String::from("World")),
    Some(String::from("NX")),
    Some(String::from("GET")),
    None,
    None,
    Ok(String::from("\"OK\""))
)]
// With NX and XX
#[case(
    "SET",
    Some(String::from("Doe")),
    Some(String::from("Test")),
    Some(String::from("NX")),
    Some(String::from("XX")),
    None,
    None,
    Err(String::from("NX/XX: syntax error"))
)]
fn test_set(
    #[case] command: &str,
    #[case] first_arg: Option<String>,
    #[case] second_arg: Option<String>,
    #[case] third_arg: Option<String>,
    #[case] fourth_arg: Option<String>,
    #[case] fifth_arg: Option<String>,
    #[case] sixth_arg: Option<String>,
    #[case] expected_response: Result<String, String>,
    mut cnxn: Connection,
) -> RedisResult<()> {
    let actual_response: Result<String, redis::RedisError> = redis::cmd(command)
        .arg(&first_arg)
        .arg(&second_arg)
        .arg(&third_arg)
        .arg(&fourth_arg)
        .arg(&fifth_arg)
        .arg(&sixth_arg)
        .query(&mut cnxn);

    println!("{:?}", actual_response);
    // Compare the two Result values using assert_eq!
    match (expected_response, actual_response) {
        (Ok(expected), Ok(actual)) => {
            assert_eq!(expected, actual);
        }
        (Err(expected_err), Err(actual_err)) => {
            assert_eq!(expected_err, actual_err.to_string());
        }
        _ => {
            panic!("Expected and actual responses do not match.");
        }
    }

    Ok(())
}

#[rstest]
// With EX
// Not Expired
#[case(
    Some(String::from("TestKey1")),
    Some(String::from("TestValue")),
    Some(String::from("EX")),
    Some(String::from("1")),
    250,
    Ok(String::from("TestValue"))
)]
// With PX
// Not Expired
#[case(
    Some(String::from("TestKey2")),
    Some(String::from("TestValue")),
    Some(String::from("PX")),
    Some(String::from("1000")),
    250,
    Ok(String::from("TestValue"))
)]
// With PX
// Expired
#[case(
    Some(String::from("TestKey3")),
    Some(String::from("TestValue")),
    Some(String::from("PX")),
    Some(String::from("500")),
    1000,
    Err(String::from("(response was nil)"))
)]
// With EXAT
// Not Expired
#[case(
    Some(String::from("TestKey4")),
    Some(String::from("TestValue")),
    Some(String::from("EXAT")),
    Some(String::from((Utc::now() + Duration::seconds(2)).timestamp().to_string())),
    1000,
    Ok(String::from("TestValue"))
)]
// With PXAT
// Expired
#[case(
    Some(String::from("TestKey5")),
    Some(String::from("TestValue")),
    Some(String::from("PXAT")),
    Some(String::from((Utc::now() + Duration::seconds(1)).timestamp_millis().to_string())),
    1500,
    Err(String::from("(response was nil)"))
)]
fn test_set_and_get_key_expiry(
    #[case] key: Option<String>,
    #[case] value: Option<String>,
    #[case] duration_arg: Option<String>,
    #[case] timespan_arg: Option<String>,
    #[case] milliseconds_delay_for: u64,
    #[case] get_expected_response: Result<String, String>,
    mut cnxn: Connection,
) -> RedisResult<()> {
    let _: Result<String, redis::RedisError> = redis::cmd("SET")
        .arg(&key)
        .arg(&value)
        .arg(&duration_arg)
        .arg(&timespan_arg)
        .query(&mut cnxn);

    std::thread::sleep(std::time::Duration::from_millis(milliseconds_delay_for));

    let get_actual_response: Result<String, redis::RedisError> =
        redis::cmd("GET").arg(&key).query(&mut cnxn);

    // Compare the two Result values using assert_eq!
    match (get_expected_response, get_actual_response) {
        (Ok(expected), Ok(actual)) => {
            assert_eq!(expected, actual);
        }
        (Err(expected_err), Err(actual_err)) => {
            assert!(actual_err.to_string().ends_with(expected_err.as_str()));
        }
        _ => {
            panic!("Expected and actual responses do not match.");
        }
    }

    Ok(())
}
