/// Asserts that encoding the left expression will lead to a success result whose stored result is `$right`.
///
/// # Parameters
/// The syntax is `ops, left => right`.
///
/// - `ops`: The `DynamicOps` to use to encode (without the `&`).
/// - `left`: The expression to encode.
/// - `right`: The expected encoded value from encoding, as a success.
#[macro_export]
macro_rules! assert_encode_success {
    ($ops:expr, $left:expr $(,)?) => {{
        let result = $crate::codec::Encode::encode_start(&$left, &$ops);
        $crate::assert_data_result_success!(result)
    }};
    ($ops:expr, $left:expr => $right:expr $(,)?) => {{
        let result = $crate::codec::Encode::encode_start(&$left, &$ops);
        $crate::assert_data_result_success!(result => $right)
    }};
}

/// Asserts that encoding the left expression will lead to an error result whose expanded message is `$right`.
///
/// # Parameters
/// The syntax is `ops, left => (partial, ) right`.
///
/// - `ops`: The `DynamicOps` to use to encode (without the `&`).
/// - `left`: The expression to encode.
/// - `partial` (optional): Assert the partial value exists and is equal to this value.
/// - `right`: The expected error message.
#[macro_export]
macro_rules! assert_encode_error {
    ($ops:expr, $left:expr) => {{
        let result = $crate::codec::Encode::encode_start(&$left, &$ops);
        $crate::assert_data_result_error!(result)
    }};
    ($ops:expr, $left:expr => $partial:expr, $right:expr $(,)?) => {{
        let result = $crate::codec::Encode::encode_start(&$left, &$ops);
        $crate::assert_data_result_error!(result => $partial, $right)
    }};
    ($ops:expr, $left:expr => $right:expr $(,)?) => {{
        let result = $crate::codec::Encode::encode_start(&$left, &$ops);
        $crate::assert_data_result_error!(result => $right)
    }};
}

/// Asserts that decoding the left expression will lead to a success result whose stored result is `$right`.
///
/// # Parameters
/// The syntax is `ops, ty, left => right`.
///
/// - `ops`: The `DynamicOps` to use to decode (without the `&`).
/// - `ty`: The type to use to decode.
/// - `left`: The value to try decoding.
/// - `right`: The expected encoded value from decoding, as a success.
#[macro_export]
macro_rules! assert_decode_success {
    ($ops:expr, $ty:ty, $left:expr $(,)?) => {{
        $crate::assert_data_result_success!(<$ty as $crate::codec::Decode>::decode(&$ops, &$left))
    }};
    ($ops:expr, $ty:ty, $left:expr => $right:expr $(,)?) => {{
        $crate::assert_data_result_success!(<$ty as $crate::codec::Decode>::decode(&$ops, &$left) => $right)
    }};
}

/// Asserts that decoding the left expression will lead to an error result whose expanded message is `$right`.
///
/// # Parameters
/// The syntax is `ops, ty, left => (partial, ) right`.
///
/// - `ops`: The `DynamicOps` to use to decode (without the `&`).
/// - `ty`: The type to use to decode.
/// - `left`: The value to try decoding.
/// - `partial` (optional): Assert the partial value exists and is equal to this value.
/// - `right`: The expected error message.
#[macro_export]
macro_rules! assert_decode_error {
    ($ops:expr, $ty:ty, $left:expr $(,)?) => {{
        let result = <$ty as $crate::codec::Decode>::decode(&$ops, &$left);
        $crate::assert_data_result_error!(result)
    }};
    ($ops:expr, $ty:ty, $left:expr => $partial:expr, $right:expr $(,)?) => {{
        let result = <$ty as $crate::codec::Decode>::decode(&$ops, &$left);
        $crate::assert_data_result_error!(result => $partial, $right)
    }};
    ($ops:expr, $ty:ty, $left:expr => $right:expr $(,)?) => {{
        let result = <$ty as $crate::codec::Decode>::decode(&$ops, &$left);
        $crate::assert_data_result_error!(result => $right)
    }};
}
