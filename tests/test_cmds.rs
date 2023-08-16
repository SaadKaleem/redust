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
fn test_redis_echo(mut cnxn: Connection) -> RedisResult<()> {
    let message = "Hello, Redis!";
    let echoed_message: String = redis::cmd("ECHO").arg(message).query(&mut cnxn)?;

    assert_eq!(echoed_message, message);
    Ok(())
}
