/// Asserts that the `$left` `DataResult` is a complete result (success) whose stored result is `$right`.
#[macro_export]
macro_rules! assert_data_result_success {
    ($left:expr $(,)?) => {{
        let result = $left;
        assert!(
            result.is_success(),
            "`DataResult` should have been a success, but got {:?}",
            result
        );
    }};
    ($left:expr => $right:expr $(,)?) => {{
        let result = $left;
        assert!(
            result.is_success(),
            "`DataResult` should have been a success, but got {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            $right,
            "`DataResult` was successful, but the values should have matched"
        );
    }};
}

/// Asserts that the `$left` `DataResult` is an error whose combined message is `$right`.
#[macro_export]
macro_rules! assert_data_result_error {
    ($left:expr $(,)?) => {{
        let result = $left;
        assert!(
            result.is_error(),
            "`DataResult` should have been an error, but got {:?}",
            result
        );
    }};
    ($left:expr => $right:expr $(,)?) => {{
        let result = $left;
        assert!(
            result.is_error(),
            "`DataResult` should have been an error, but got {:?}",
            result
        );
        assert_eq!(
            result.message().unwrap(),
            $right.to_string(),
            "`DataResult` was successful, but the error messages should have matched"
        );
    }};
}
