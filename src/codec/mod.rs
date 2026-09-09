mod either;
mod list;
mod map;
mod option;
mod primitive;

use std::borrow::Cow;
use crate::builder::RecordBuilder;
use crate::dynamic_ops::{DynamicOps, MapLike};
use crate::lifecycle::Lifecycle;
use crate::{DataError, DataResult};
pub use list::BoundedVec;
pub use option::{OptionalFieldDecode, OptionalFieldEncode};
use thiserror::Error;
use crate::json_ops::JsonOpsError;

/// A trait for something that can be encoded by a [`DynamicOps`] to its format.
pub trait Encode {
    /// Encodes this value to a value represented by the provided [`DynamicOps`] with the given prefix.
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value>;

    /// Encodes this value to a value represented by the provided [`DynamicOps`] without a prefix.
    fn encode_start<O: DynamicOps>(&self, ops: &O) -> DataResult<O::Value> {
        self.encode(ops, ops.empty())
    }

    /// Encodes this value to a map by adding a field, whose:
    /// - key is the field's `name`.
    /// - value is the encoded value represented by the provided [`DynamicOps`].
    fn encode_field<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        prefix: B,
        ops: &O,
        name: impl Into<String>,
    ) -> B {
        prefix.add_result(name, self.encode_start(ops))
    }

    /// Encodes this value to a map by adding a defaulted field, whose:
    /// - key is the field's `name`.
    /// - value is the encoded value represented by the provided [`DynamicOps`].
    ///
    /// The field may not be encoded if `default` == `*self`.
    fn encode_defaulted_field<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        prefix: B,
        ops: &O,
        name: &'static str,
        default: Self,
    ) -> B
    where
        Self: PartialEq + Sized,
    {
        if default != *self {
            self.encode_field(prefix, ops, name)
        } else {
            prefix
        }
    }
}

/// A trait for something that can be decoded from a value represented by a [`DynamicOps`].
pub trait Decode: Sized {
    /// Decodes a value of this type from a value represented by the provided [`DynamicOps`].
    fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self>;

    /// Decodes a value of this type from a map by decoding one of its fields, whose:
    /// - key is the field's `name`.
    /// - value is the value represented by a [`DynamicsOps`] that is meant to be decoded.
    fn decode_field<O: DynamicOps>(
        input: &mut impl MapLike<Value = O::Value>,
        ops: &O,
        name: &'static str,
    ) -> DataResult<Self> {
        let Some(value) = input.remove(name) else {
            return DataResult::error(BuiltInError::NoKey(name.into()));
        };
        Self::decode(ops, value)
    }

    /// Decodes a value of this type from a map by decoding one of its defaulted fields, whose:
    /// - key is the field's `name`.
    /// - value is the value represented by a [`DynamicsOps`] that is meant to be decoded.
    ///
    /// If a value could not be decoded, the `default` value is returned.
    ///
    /// This method has an extra `lenient` parameter. If it is `true`, errors
    /// while trying to decode an explicit value are *ignored*, decoding the default instead.
    fn decode_defaulted_field<O: DynamicOps>(
        input: &mut impl MapLike<Value = O::Value>,
        ops: &O,
        name: &'static str,
        default: Self,
        lenient: bool,
    ) -> DataResult<Self> {
        let decoded_option = Option::decode_optional_field::<O>(input, ops, name, lenient);
        decoded_option.map(|o| o.unwrap_or(default))
    }
}

/// A trait for something which can be added to a map builder (usually with more than 1 field).
///
/// This is used mostly for encoding structures, and this trait is usually adapted to work with [`Encode`].
pub trait MapEncode {
    /// Encodes this value to a map by adding one or more fields.
    fn map_encode<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        ops: &O,
        prefix: B,
    ) -> B;
}

impl<T: MapEncode> Encode for T {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        self.map_encode(ops, ops.map_builder()).build(prefix)
    }
}

/// A trait for something which can be decoded from a [`MapLike`] (usually with more than 1 field).
///
/// This is used mostly for encoding structures, and this trait is usually adapted to work with [`Decode`].
pub trait MapDecode: Sized {
    /// Decodes a value of this type from a map by decoding one or more fields.
    fn map_decode<O: DynamicOps>(
        ops: &O,
        input: impl MapLike<Value = O::Value>,
    ) -> DataResult<Self>;
}

impl<T: MapDecode> Decode for T {
    fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        ops.try_map(input)
            .with_lifecycle(Lifecycle::Stable)
            .and_then(|map| T::map_decode(ops, map))
    }
}

#[derive(Error, Debug)]
pub enum BuiltInError {
    // Dynamic ops errors
    #[error("Do not know how to append a primitive value {0} to {1}")]
    DoNotKnowHowToAppendPrimitive(Box<str>, Box<str>),
    #[error("Some elements are not {0}: {1}")]
    SomeElementsAreDifferent(&'static str, Box<str>),
    
    #[error("{0}")]
    Json(JsonOpsError),

    // Codec errors
    #[error("No key {0} in map MapLike[{{}}]")]
    NoKey(Cow<'static, str>),
    #[error("Could not fit {0} to {1}: {2}")]
    CouldNotFit(&'static str, &'static str, Box<str>),
    #[error("List is too short: {size}, expected range [{min_size}-{max_size}]")]
    TooShortList {
        size: usize,
        min_size: usize,
        max_size: usize,
    },
    #[error("List is too long: {size}, expected range [{min_size}-{max_size}]")]
    TooLongList {
        size: usize,
        min_size: usize,
        max_size: usize,
    },
    #[error("Duplicate entry for key: '{0}'")]
    DuplicateEntry(Box<str>),
    #[error("Failed to parse either. First: {0}; Second: {1}")]
    FailedToParseEither(Box<DataError>, Box<DataError>),
}
