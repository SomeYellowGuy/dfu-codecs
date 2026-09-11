use crate::DynamicOps;
use crate::codec::core::primitive::sealed::Primitive;
use crate::codec::{BuiltInError, Decode, Encode};
use crate::data_result::DataResult;

mod sealed {
    use crate::DynamicOps;
    use crate::data_result::DataResult;

    /// Sealed trait to easily implement `Encode` and `Decode` for
    /// primitive DFU types.
    pub trait Primitive: Sized {
        fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value;
        fn primitive_decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self>;
    }
}

macro_rules! primitive_blanket_impl {
    ( $($ty:ty),+ ) => {
        $(
            impl Encode for $ty {
                #[inline]
                fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
                    ops.merge_to_primitive(prefix, self.primitive_encode(ops))
                }

                #[inline]
                fn encode_start<O: DynamicOps>(&self, ops: &O) -> DataResult<O::Value> {
                    DataResult::success(self.primitive_encode(ops))
                }
            }

            impl Decode for $ty {
                #[inline]
                fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
                    <$ty>::primitive_decode(ops, input)
                }
            }
        )+
    };
}

macro_rules! impl_number {
    ($ty:ty, $create_func:ident, $try_func:ident) => {
        impl Primitive for $ty {
            #[inline]
            fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
                ops.$create_func(*self)
            }

            #[inline]
            fn primitive_decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
                ops.$try_func(input)
            }
        }

        primitive_blanket_impl!($ty);
    };
}

macro_rules! impl_number_and_unsigned {
    ($ty:ty, $uty:ty, $create_func:ident, $try_func:ident) => {
        impl_number!($ty, $create_func, $try_func);

        // Unsigned type
        impl Encode for $uty {
            #[inline]
            fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
                <$ty>::try_from(*self).map_or_else(
                    |_| {
                        DataResult::error(BuiltInError::CouldNotFit(
                            stringify!($uty),
                            stringify!($ty),
                            self.to_string().into(),
                        ))
                    },
                    |i| i.encode(ops, prefix),
                )
            }
        }
        impl Decode for $uty {
            #[inline]
            fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
                <$ty>::decode(ops, input).and_then(|i| {
                    <$uty>::try_from(i).map_or_else(
                        |_| {
                            DataResult::error(BuiltInError::CouldNotFit(
                                stringify!($ty),
                                stringify!($uty),
                                i.to_string().into(),
                            ))
                        },
                        DataResult::success,
                    )
                })
            }
        }
    };
}

impl_number_and_unsigned!(i8, u8, byte, try_byte);
impl_number_and_unsigned!(i16, u16, short, try_short);
impl_number_and_unsigned!(i32, u32, int, try_int);
impl_number_and_unsigned!(i64, u64, long, try_long);

impl_number!(f32, float, try_float);
impl_number!(f64, double, try_double);

impl Primitive for bool {
    #[inline]
    fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
        ops.bool(*self)
    }

    #[inline]
    fn primitive_decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        ops.try_bool(input)
    }
}

impl Primitive for String {
    #[inline]
    fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
        ops.string(self.clone())
    }

    #[inline]
    fn primitive_decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        ops.try_string(input)
    }
}

primitive_blanket_impl!(bool, String);

macro_rules! wrapper {
    ($stream:ident, $ty:ty, $create_func:ident, $get_func:ident) => {
        #[doc = concat!("A wrapper of a [`Vec<", stringify!($ty), ">`] that has a built-in Encode and Decode implementation.")]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $stream(pub Vec<$ty>);

        impl From<Vec<$ty>> for $stream {
            fn from(value: Vec<$ty>) -> Self {
                Self(value)
            }
        }

        impl From<$stream> for Vec<$ty> {
            fn from(value: $stream) -> Vec<$ty> {
                value.0
            }
        }

        impl Primitive for $stream {
            fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
                ops.$create_func(&self.0)
            }

            fn primitive_decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
                ops.$get_func(input).map($stream)
            }
        }
        primitive_blanket_impl!($stream);
    };
}

wrapper!(ByteBuffer, i8, byte_buffer, try_byte_list);
wrapper!(IntStream, i32, int_stream, try_int_list);
wrapper!(LongStream, i64, long_stream, try_long_list);

#[cfg(test)]
mod tests {
    use crate::codec::core::primitive::{ByteBuffer, IntStream, LongStream};
    use crate::json_ops::JsonOps;
    use crate::{assert_decode_error, assert_decode_success, assert_encode_success};
    use serde_json::json;

    #[test]
    fn encoding() {
        assert_encode_success!(JsonOps, 75 => json!(75));
        assert_encode_success!(JsonOps, -103i8 => json!(-103));
        assert_encode_success!(JsonOps, -123_847_234 => json!(-123_847_234));
        assert_encode_success!(JsonOps, false => json!(false));
        assert_encode_success!(JsonOps, "Hello, world!".to_string() => json!("Hello, world!"));
        assert_encode_success!(JsonOps, String::new() => json!(""));

        assert_encode_success!(JsonOps, ByteBuffer::from(vec![1, 2, 3]) => json!([1, 2, 3]));
        assert_encode_success!(
            JsonOps,
            IntStream::from(vec![3, 6, 9, 11, 15]) =>
            json!([3, 6, 9, 11, 15])
        );
        assert_encode_success!(
            JsonOps,
            LongStream::from(vec![4, 6, 9, 12]) =>
            json!([4, 6, 9, 12])
        );
    }

    #[test]
    fn decoding() {
        assert_decode_success!(JsonOps, i32, json!(8) => 8);
        assert_decode_success!(JsonOps, i32, json!(4.5) => 4);
        assert_decode_success!(JsonOps, i64, json!(2412.234) => 2412);
        assert_decode_error!(JsonOps, u32, json!(-45) => "Could not fit i32 to u32: -45");
        assert_decode_success!(JsonOps, i8, json!(1000) => -24);
        assert_decode_error!(JsonOps, bool, json!("hello") => "Not a boolean: \"hello\"");
        assert_decode_error!(JsonOps, bool, json!(0) => "Not a boolean: 0");
        assert_decode_success!(JsonOps, String, json!("cool") => "cool");
        assert_decode_error!(JsonOps, String, json!(1) => "Not a string: 1");

        assert_decode_success!(JsonOps, IntStream, json!([1, 2, 3]) => IntStream::from(vec![1, 2, 3]));
        assert_decode_success!(JsonOps, LongStream, json!([]) => LongStream::from(vec![]));
        assert_decode_error!(JsonOps, ByteBuffer, json!("Sample Text") => "Not a JSON array: \"Sample Text\"");
    }
}
