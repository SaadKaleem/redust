use redis::{Client, Connection, RedisResult};
use rstest::fixture;
use rstest::rstest;

#[fixture]
fn cnxn() -> Connection {
    let client = Client::open("redis://127.0.0.1:6666/").unwrap();
    let cnxn = client.get_connection().unwrap();
    cnxn
}

#[rstest]
fn test_ping(mut cnxn: Connection) -> RedisResult<()> {
    let ping_response: String = redis::cmd("PING").query(&mut cnxn)?;

    assert_eq!(ping_response, "PONG");
    Ok(())
}

#[rstest]
#[case("ECHO", Some("Hello, World!".to_string()))]
// Missing Argument
#[case("ECHO", None)]
fn test_echo(
    #[case] command: &str,
    #[case] message: Option<String>,
    mut cnxn: Connection,
) -> RedisResult<()> {

    let response: Result<String, redis::RedisError> =
        redis::cmd(command).arg(&message).query(&mut cnxn);

    match message {
        Some(message) => assert_eq!(response.unwrap(), message),
        None => assert_eq!(
            response.err().unwrap().to_string(),
            "ERR:: Missing Argument".to_string()
        ),
    }

    Ok(())
}
