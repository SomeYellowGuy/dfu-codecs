/// Provides a fallible/infallible encode conversion from a first type to a second type via `backward`.
///
/// You probably do not need to use this directly.
#[macro_export]
macro_rules! encode_impl {
    (infallible $first_type:ty, $second_type:ty, $backward:path) => {
        impl $crate::codec::Encode for $second_type {
            fn encode<O: $crate::DynamicOps>(
                &self,
                ops: &O,
                prefix: Option<O::Value>,
            ) -> $crate::DataResult<O::Value> {
                <$first_type as $crate::codec::Encode>::encode(&$backward(self), ops, prefix)
            }
        }
    };

    (fallible $first_type:ty, $second_type:ty, $backward:path) => {
        impl $crate::codec::Encode for $second_type {
            fn encode<O: $crate::DynamicOps>(
                &self,
                ops: &O,
                prefix: Option<O::Value>,
            ) -> $crate::DataResult<O::Value> {
                $backward(self)
                    .and_then(|m| <$second_type as $crate::codec::Encode>::encode(&m, ops, prefix))
            }
        }
    };
}

/// Provides a fallible/infallible decode conversion from a first type to a second type via `forward`.
///
/// You probably do not need to use this directly.
#[macro_export]
macro_rules! decode_impl {
    (infallible $first_type:ty, $second_type:ty, $forward:path) => {
        impl $crate::codec::Decode for $second_type {
            fn decode<O: $crate::DynamicOps>(
                ops: &O,
                input: &O::Value,
            ) -> $crate::DataResult<Self> {
                <$first_type as $crate::codec::Decode>::decode(ops, input).map($forward)
            }
        }
    };

    (fallible $first_type:ty, $second_type:ty, $forward:path) => {
        impl $crate::codec::Decode for $second_type {
            fn decode<O: $crate::DynamicOps>(
                ops: &O,
                input: &O::Value,
            ) -> $crate::DataResult<Self> {
                <$first_type as $crate::codec::Decode>::decode(ops, input).and_then($forward)
            }
        }
    };
}

/// Provides easy `xmap`-like `Encode` and `Decode` implementations of a *second type*
/// by using the already-existing implementations of a *first type*.
///
/// The macro is written as `xmap_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the infallible conversion `fn(first) -> second` (for decoding).
/// - `backward` is the infallible conversion `fn(&second) -> first` (for encoding).
#[macro_export]
macro_rules! xmap_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(infallible $first_type, $second_type, $backward);
        $crate::decode_impl!(infallible $first_type, $second_type, $forward);
    };
}

/// Provides easy `comapFlatMap`-like `Encode` and `Decode` implementations of a *second type*
/// by using the already-existing implementations of a *first type*.
///
/// The macro is written as `comap_flat_map_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the fallible conversion `fn(first) -> DataResult<second>` (for decoding).
/// - `backward` is the infallible conversion `fn(&second) -> first` (for encoding).
#[macro_export]
macro_rules! comap_flat_map_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(infallible $first_type, $second_type, $backward);
        $crate::decode_impl!(fallible $first_type, $second_type, $forward);
    };
}

/// Provides easy `flatComapMap`-like `Encode` and `Decode` implementations of a *second type*
/// by using the already-existing implementations of a *first type*.
///
/// The macro is written as `flat_comap_map_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the infallible conversion `fn(first) -> second` (for decoding).
/// - `backward` is the fallible conversion `fn(&second) -> DataResult<first>` (for encoding).
#[macro_export]
macro_rules! flat_comap_map_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(fallible $first_type, $second_type, $backward);
        $crate::decode_impl!(infallible $first_type, $second_type, $forward);
    };
}

/// Provides easy `flatXmap`-like `Encode` and `Decode` implementations of a *second type*
/// by using the already-existing implementations of a *first type*.
///
/// The macro is written as `flat_xmap_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the fallible conversion `fn(first) -> DataResult<second>` (for decoding).
/// - `backward` is the fallible conversion `fn(&second) -> DataResult<first>` (for encoding).
#[macro_export]
macro_rules! flat_xmap_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(fallible $first_type, $second_type, $backward);
        $crate::decode_impl!(fallible $first_type, $second_type, $forward);
    };
}
