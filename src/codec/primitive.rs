use crate::codec::primitive::sealed::Primitive;
use crate::codec::{Decode, Encode};
use crate::{DataResult, DynamicOps};
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Could not fit {0} to {1}: {2}")]
struct CouldNotFitError(&'static str, &'static str, String);

mod sealed {
    use super::{DataResult, DynamicOps};

    /// Sealed trait to easily implement `Encode` and `Decode` for
    /// primitive DFU types.
    pub trait Primitive: Sized {
        fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value;
        fn primitive_decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self>;
    }
}

macro_rules! primitive_blanket_impl {
    ( $($ty:ty),+ ) => {
        $(
            impl Encode for $ty {
            fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
                    ops.merge_to_primitive(prefix, self.primitive_encode(ops))
                }
            }

            impl Decode for $ty {
                fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
                    <$ty>::primitive_decode(ops, input)
                }
            }
        )+
    };
}

macro_rules! impl_number {
    ($ty:ty, $create_func:ident) => {
        impl Primitive for $ty {
            fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
                ops.$create_func(*self)
            }

            fn primitive_decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
                ops.try_number(input).map(|n| <$ty>::from(n))
            }
        }

        primitive_blanket_impl!($ty);
    };
}

macro_rules! impl_number_and_unsigned {
    ($ty:ty, $uty:ty, $create_func:ident) => {
        impl_number!($ty, $create_func);

        // Unsigned type
        impl Encode for $uty {
            fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
                <$ty>::try_from(*self).map_or_else(
                    |_| {
                        DataResult::error(CouldNotFitError(
                            stringify!($uty),
                            stringify!($ty),
                            self.to_string(),
                        ))
                    },
                    |i| i.encode(ops, prefix),
                )
            }
        }
        impl Decode for $uty {
            fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
                <$ty>::decode(ops, input).and_then(|i| {
                    <$uty>::try_from(i).map_or_else(
                        |_| {
                            DataResult::error(CouldNotFitError(
                                stringify!($ty),
                                stringify!($uty),
                                i.to_string(),
                            ))
                        },
                        DataResult::success,
                    )
                })
            }
        }
    };
}

impl_number_and_unsigned!(i8, u8, byte);
impl_number_and_unsigned!(i16, u16, short);
impl_number_and_unsigned!(i32, u32, int);
impl_number_and_unsigned!(i64, u64, long);

impl_number!(f32, float);
impl_number!(f64, double);

impl Primitive for bool {
    fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
        ops.bool(*self)
    }

    fn primitive_decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        ops.try_bool(input)
    }
}

impl Primitive for String {
    fn primitive_encode<O: DynamicOps>(&self, ops: &O) -> O::Value {
        ops.string(self.clone())
    }

    fn primitive_decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        ops.try_string(input)
    }
}

primitive_blanket_impl!(bool, String);

macro_rules! wrapper {
    ($stream:ident, $ty:ty, $create_func:ident, $get_func:ident) => {
        #[doc = concat!("A wrapper of a [`Vec<", stringify!($ty), ">`] that has a built-in Encode and Decode implementation.")]
        #[derive(Debug, Clone)]
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

            fn primitive_decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
                ops.$get_func(input).map($stream)
            }
        }
        primitive_blanket_impl!($stream);
    };
}

wrapper!(ByteBuffer, i8, byte_buffer, try_byte_list);
wrapper!(IntStream, i32, int_stream, try_int_list);
wrapper!(LongStream, i64, long_stream, try_long_list);
