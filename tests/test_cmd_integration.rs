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
