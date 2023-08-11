use redust::protocol_handler::{deserialize_buffer, serialize_data, BulkStringData, RESPType};
use rstest::rstest;

#[rstest]
// Simple String Cases
#[case(b"+PING", (None, 0usize))]
#[case(b"+OK\r\n", (Some(RESPType::SimpleString("OK".to_string())), 5usize))]
#[case(b"+OK\r\n+Next", (Some(RESPType::SimpleString("OK".to_string())), 5usize))]
// Error Test Cases
#[case(b"-ERR", (None, 0usize))]
#[case(b"-ERR\r\n", (Some(RESPType::Error("ERR".to_string())), 6usize))]
#[case(b"-ERR\r\n+Partial", (Some(RESPType::Error("ERR".to_string())), 6usize))]
// Integer Test Cases
#[case(b":20", (None, 0usize))]
#[case(b":20\r\n", (Some(RESPType::Integer(20)), 5usize))]
#[case(b":20\r\n+PING", (Some(RESPType::Integer(20)), 5usize))]
// Bulk String Test Cases
#[case(b"$5\r\nhello", (None, 0usize))]
#[case(b"$0\r\n\r\n", (Some(RESPType::BulkString(Some(BulkStringData{text: "".to_string(), prefix_length: 0}))), 6usize))]
#[case(b"$-1\r\n", (Some(RESPType::BulkString(None)), 5usize))]
#[case(b"$4\r\ntest\r\n", (Some(RESPType::BulkString(Some(BulkStringData{text: "test".to_string(), prefix_length: 4}))), 10usize))]
#[case(b"$4\r\ntest\r\n+Next", (Some(RESPType::BulkString(Some(BulkStringData{text: "test".to_string(), prefix_length: 4}))), 10usize))]
// Array Test Cases
#[case(b"*0", (None, 0usize))]
#[case(b"*0\r\n", (Some(RESPType::Array(vec![])),4usize))]
#[case(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n", (Some(RESPType::Array(
vec![
    RESPType::BulkString(Some(BulkStringData{text: "hello".to_string(), prefix_length: 5})),
    RESPType::BulkString(Some(BulkStringData{text: "world".to_string(), prefix_length: 5}))
    ])),
    26usize)
)]
#[case(b"*2\r\n*2\r\n+Hello\r\n$5\r\nWorld\r\n*3\r\n:1\r\n:2\r\n:3\r\n", (Some(RESPType::Array(
    vec![
        RESPType::Array(
            vec![
                RESPType::SimpleString("Hello".to_string()),
                RESPType::BulkString(Some(BulkStringData{text: "World".to_string(), prefix_length: 5}))]
        ),
        RESPType::Array(
            vec![
                RESPType::Integer(1),
                RESPType::Integer(2),
                RESPType::Integer(3)
                ]
        )
        ])),
        43usize)
)]

fn deserialize_test(#[case] input: &[u8], #[case] expected: (Option<RESPType>, usize)) {
    let actual: (Option<RESPType>, usize) = deserialize_buffer(input);
    assert_eq!(expected.0, actual.0);
    assert_eq!(expected.1, actual.1);
}

#[rstest]
// Simple String Cases
#[case(RESPType::SimpleString("".to_string()), Some(b"+\r\n".to_vec()))]
#[case(RESPType::SimpleString("OK".to_string()), Some(b"+OK\r\n".to_vec()))]
// Error Test Cases
#[case(RESPType::Error("".to_string()), Some(b"-\r\n".to_vec()))]
#[case(RESPType::Error("ERR".to_string()), Some(b"-ERR\r\n".to_vec()))]
// Integer Test Cases
#[case(RESPType::Integer(-1), Some(b":-1\r\n".to_vec()))]
#[case(RESPType::Integer(20), Some(b":20\r\n".to_vec()))]
// Bulk String Test Cases
#[case(RESPType::BulkString(Some(BulkStringData{text: "".to_string(), prefix_length: 0})), Some(b"$0\r\n\r\n".to_vec()))]
#[case(RESPType::BulkString(None), Some(b"$-1\r\n".to_vec()))]
#[case(RESPType::BulkString(Some(BulkStringData{text: "test".to_string(), prefix_length: 4})), Some(b"$4\r\ntest\r\n".to_vec()))]
// Array Test Cases
#[case(RESPType::Array(
    vec![
        RESPType::BulkString(Some(BulkStringData{text: "hello".to_string(), prefix_length: 5})),
        RESPType::BulkString(Some(BulkStringData{text: "world".to_string(), prefix_length: 5}))
        ]), 
        Some(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".to_vec())
)]
#[case(RESPType::Array(
    vec![
        RESPType::Array(
            vec![
                RESPType::SimpleString("Hello".to_string()),
                RESPType::BulkString(Some(BulkStringData{text: "World".to_string(), prefix_length: 5}))]
        ),
        RESPType::Array(
            vec![
                RESPType::Integer(1),
                RESPType::Integer(2),
                RESPType::Integer(3)
                ]
        )
        ]),
        Some(b"*2\r\n*2\r\n+Hello\r\n$5\r\nWorld\r\n*3\r\n:1\r\n:2\r\n:3\r\n".to_vec())
)]
fn serialize_test(#[case] data: RESPType, #[case] expected: Option<Vec<u8>>) {
    let actual: Option<Vec<u8>> = serialize_data(&data);
    assert_eq!(expected, actual);
}
