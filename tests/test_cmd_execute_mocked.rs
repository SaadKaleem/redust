use mockall::predicate::{eq, ne};
use predicates::ord::EqPredicate;
use redust::MockConnectionBase;
use redust::{cmd::Echo, cmd::Ping, RESPType};
use rstest::rstest;

/// Ping Execute Command
/// Assumption: Good Connection
#[rstest]
// Equal to
#[case(None, eq(RESPType::SimpleString("\"PONG\"".to_string())))]
// NOT Equal to
#[case(Some("HELLO WORLD".to_string()), ne(RESPType::SimpleString("\"WORLD HELLO\"".to_string())))]
#[tokio::test]
async fn test_ping_execute_cnxn_ok(
    #[case] ping_msg: Option<String>,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let ping_cmd = Ping::new(ping_msg);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_x| Ok(()));

    // Call the function to test
    let result = ping_cmd.execute(&mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Ping Execute Command
/// Assumption: Bad Connection (reset)
#[rstest]
// Equal to
#[case(None, eq(RESPType::SimpleString("\"PONG\"".to_string())))]
// NOT Equal to
#[case(Some("HELLO WORLD".to_string()), ne(RESPType::SimpleString("\"WORLD HELLO\"".to_string())))]
#[tokio::test]
async fn test_ping_execute_cnxn_err(
    #[case] ping_msg: Option<String>,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let ping_cmd = Ping::new(ping_msg);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_x| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::ConnectionReset,
                "Connection Reset",
            ))
        });

    // Call the function to test
    let result = ping_cmd.execute(&mut mock_cnxn).await;
    assert!(result.is_err());
}

/// Echo Execute Command
/// Assumption: Good Connection
#[rstest]
// Equal to
#[case("HELLO".to_string(), eq(RESPType::SimpleString("\"HELLO\"".to_string())))]
// NOT Equal to
#[case("HELLO".to_string(), ne(RESPType::SimpleString("\"HEY\"".to_string())))]
#[tokio::test]
async fn test_echo_execute_cnxn_ok(
    #[case] echo_msg: String,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let echo_cmd = Echo::new(echo_msg);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_x| Ok(()));

    // Call the function to test
    let result = echo_cmd.execute(&mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Echo Execute Command
/// Assumption: Bad Connection
#[rstest]
// Equal to
#[case("HELLO".to_string(), eq(RESPType::SimpleString("\"HELLO\"".to_string())))]
// NOT Equal to
#[case("HELLO".to_string(), ne(RESPType::SimpleString("\"HEY\"".to_string())))]
#[tokio::test]
async fn test_echo_execute_cnxn_err(
    #[case] echo_msg: String,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let echo_cmd = Echo::new(echo_msg);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_x| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::ConnectionReset,
                "Connection Reset",
            ))
        });

    // Call the function to test
    let result = echo_cmd.execute(&mut mock_cnxn).await;
    assert!(result.is_err());
}
