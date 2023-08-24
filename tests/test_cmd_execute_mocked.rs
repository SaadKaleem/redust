use mockall::predicate::{eq, ne};
use predicates::ord::EqPredicate;
use redust::protocol_handler::BulkStringData;
use redust::DataType;
use redust::{cmd::Echo, cmd::Exists, cmd::Get, cmd::Ping, cmd::Set, RESPType};
use redust::{MockConnectionBase, MockSharedStoreBase};
use rstest::rstest;

/// Ping Execute Command
///
/// Assumption:
/// 1. Good Connection
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
        .returning(|_| Ok(()));

    // Call the function to test
    let result = ping_cmd.execute(&mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Ping Execute Command
///
/// Assumption:
/// 1. Bad Connection (reset)
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
        .returning(|_| {
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
///
/// Assumption:
/// 1. Good Connection
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
        .returning(|_| Ok(()));

    // Call the function to test
    let result = echo_cmd.execute(&mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Echo Execute Command
///
/// Assumption:
/// 1. Bad Connection
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

/// Get Execute Command
///
/// Assumption:
/// 1. No value exists at the input key
/// 2. Good Connection
#[rstest]
// Equal to
#[case("John".to_string(), eq(RESPType::BulkString(None)))]
// Not Equal to
#[case("John".to_string(), ne(RESPType::SimpleString("\"OK\"".to_string())))]
#[tokio::test]
async fn test_get_execute_no_key_value_cnxn_ok(
    #[case] key: String,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let get_cmd = Get::new(key.clone());

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    mock_shared_store
        .expect_get()
        .with(eq(key))
        .times(1)
        .returning(|_| None);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| Ok(()));

    // Call the function to test
    let result = get_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Get Execute Command
///
/// Assumption:
/// 1. Value exists at the input key
/// 2. Good Connection
#[rstest]
// Equal to
#[case("John".to_string(), eq(RESPType::BulkString(Some(BulkStringData{text:"\"Doe\"".to_string(), prefix_length: 5}))))]
// Not Equal to
#[case("John".to_string(), ne(RESPType::SimpleString("\"Doe\"".to_string())))]
#[tokio::test]
async fn test_get_execute_key_value_exists_cnxn_ok(
    #[case] key: String,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let get_cmd = Get::new(key.clone());

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    mock_shared_store
        .expect_get()
        .with(eq(key))
        .times(1)
        .returning(|_| Some(DataType::String("\"Doe\"".to_string())));

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| Ok(()));

    // Call the function to test
    let result = get_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Set Execute Command
///
/// Assumption:
/// 1. No previous value exists at the input key
/// 2. Good Connection
#[rstest]
// Equal to
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, false, false, false, eq(RESPType::SimpleString("\"OK\"".to_string())))]
// Not Equal to
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, false, false, false, ne(RESPType::SimpleString("\"TEST\"".to_string())))]
#[tokio::test]
async fn test_set_execute_no_prev_value_cnxn_ok(
    #[case] key: String,
    #[case] value: redust::DataType,
    #[case] duration: Option<chrono::Duration>,
    #[case] nx: bool,
    #[case] xx: bool,
    #[case] get: bool,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let set_cmd = Set::new(key.clone(), value.clone(), duration, nx, xx, get);

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    mock_shared_store
        .expect_set()
        .with(eq(key), eq(value), eq(duration), eq(nx), eq(xx))
        .times(1)
        .returning(|_, _, _, _, _| Ok(None));

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| Ok(()));

    // Call the function to test
    let result = set_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Set Execute Command
///
/// Assumption:
/// 1. Previous value of "HELLO, WORLD" exists at the input key
/// 2. Good Connection
#[rstest]
// Equal to, with GET = true
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, false, false, true, eq(RESPType::BulkString(Some(BulkStringData{text:"\"HELLO, WORLD\"".to_string(), prefix_length: 14}))))]
// Equal to, with GET = false
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, false, false, false, eq(RESPType::SimpleString("\"OK\"".to_string())))]
// Not Equal to, with GET = true
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, false, false, true, ne(RESPType::SimpleString("\"OK\"".to_string())))]
#[tokio::test]
async fn test_set_execute_prev_value_exists_cnxn_ok(
    #[case] key: String,
    #[case] value: redust::DataType,
    #[case] duration: Option<chrono::Duration>,
    #[case] nx: bool,
    #[case] xx: bool,
    #[case] get: bool,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let set_cmd = Set::new(key.clone(), value.clone(), duration, nx, xx, get);

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    mock_shared_store
        .expect_set()
        .with(eq(key), eq(value), eq(duration), eq(nx), eq(xx))
        .times(1)
        .returning(|_, _, _, _, _| Ok(Some(DataType::String("HELLO, WORLD".to_string()))));

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| Ok(()));

    // Call the function to test
    let result = set_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Set Execute Command
///
/// Assumption:
/// 1. Parse Error occurs at the Data Store
/// 2. Good Connection
#[rstest]
// Although the NX and XX values don't matter since the Data Store is mocked.
// Equal to, NX = true, XX = false
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, true, false, true, eq(RESPType::Error("syntax error".to_string())))]
// Equal to, NX = true, XX = true
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, true, true, false, ne(RESPType::SimpleString("\"OK\"".to_string())))]
#[tokio::test]
async fn test_set_execute_data_store_err_cnxn_ok(
    #[case] key: String,
    #[case] value: redust::DataType,
    #[case] duration: Option<chrono::Duration>,
    #[case] nx: bool,
    #[case] xx: bool,
    #[case] get: bool,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let set_cmd = Set::new(key.clone(), value.clone(), duration, nx, xx, get);

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    mock_shared_store
        .expect_set()
        .with(eq(key), eq(value), eq(duration), eq(nx), eq(xx))
        .times(1)
        .returning(|_, _, _, _, _| {
            Err(redust::cmd::ParseError::SyntaxError(
                "syntax error".to_string(),
            ))
        });

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| Ok(()));

    // Call the function to test
    let result = set_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// Set Execute Command
///
/// Assumption:
/// 1. Parse Error occurs at the Data Store
/// 2. Bad Connection (reset)
#[rstest]
// Although the NX and XX values don't matter since the Data Store is mocked.
// Equal to, NX = true, XX = false
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, true, false, true, eq(RESPType::Error("syntax error".to_string())))]
// Equal to, NX = true, XX = true
#[case("John".to_string(), redust::DataType::String("Doe".to_string()), None, true, true, false, ne(RESPType::SimpleString("\"OK\"".to_string())))]
#[tokio::test]
async fn test_set_execute_data_store_err_cnxn_err(
    #[case] key: String,
    #[case] value: redust::DataType,
    #[case] duration: Option<chrono::Duration>,
    #[case] nx: bool,
    #[case] xx: bool,
    #[case] get: bool,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let set_cmd = Set::new(key.clone(), value.clone(), duration, nx, xx, get);

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    mock_shared_store
        .expect_set()
        .with(eq(key), eq(value), eq(duration), eq(nx), eq(xx))
        .times(1)
        .returning(|_, _, _, _, _| {
            Err(redust::cmd::ParseError::SyntaxError(
                "syntax error".to_string(),
            ))
        });

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::ConnectionReset,
                "Connection Reset",
            ))
        });

    // Call the function to test
    let result = set_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_err());
}

/// EXISTS Execute Command
///
/// Assumption:
/// 1. All keys Exist
/// 2. Good Connection
#[rstest]
// Equal to
#[case(vec!["Key1".to_string(), "Key2".to_string()], eq(RESPType::Integer(2)))]
// Not Equal to
#[case(vec!["Key1".to_string(), "Key2".to_string()], ne(RESPType::Integer(1)))]
#[tokio::test]
async fn test_exists_execute_all_keys_exist_cnxn_ok(
    #[case] keys: Vec<String>,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let exists_cmd = Exists::new(keys.clone());

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    let keys_length: u64 = keys.len() as u64;

    mock_shared_store
        .expect_exists()
        .with(eq(keys))
        .times(1)
        .returning(move |_| keys_length);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| Ok(()));

    // Call the function to test
    let result = exists_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_ok());
}

/// EXISTS Execute Command
///
/// Assumption:
/// 1. keys.len() - 1 e.g. If two keys, then only one key exists.
/// 2. Bad Connection
#[rstest]
// Equal to
#[case(vec!["Key1".to_string(), "Key2".to_string()], ne(RESPType::Integer(2)))]
// Not Equal to
#[case(vec!["Key1".to_string(), "Key2".to_string()], eq(RESPType::Integer(1)))]
#[tokio::test]
async fn test_exists_execute_all_keys_exist_cnxn_err(
    #[case] keys: Vec<String>,
    #[case] expected_input_cnxn_write_frame: EqPredicate<RESPType>,
) {
    // Create the Command instance
    let exists_cmd = Exists::new(keys.clone());

    // Create the Shared Store Mock
    let mut mock_shared_store = MockSharedStoreBase::new();

    let keys_length: u64 = (keys.len() as u64) - 1;

    mock_shared_store
        .expect_exists()
        .with(eq(keys))
        .times(1)
        .returning(move |_| keys_length);

    // Create the Connection Mock
    let mut mock_cnxn = MockConnectionBase::new();

    // Add the expected conditions, to assert for the Mocked Connection
    mock_cnxn
        .expect_write_frame()
        .with(expected_input_cnxn_write_frame)
        .times(1)
        .returning(|_| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::ConnectionReset,
                "Connection Reset",
            ))
        });

    // Call the function to test
    let result = exists_cmd.execute(&mock_shared_store, &mut mock_cnxn).await;
    assert!(result.is_err());
}
