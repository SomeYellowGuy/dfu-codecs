use crate::builder::RecordBuilder;
use crate::codec::{Decode, Encode};
use crate::data_result::DataResultKind;
use crate::dynamic_ops::MapLike;
use crate::{DataResult, DynamicOps};

/// A trait for something which can be added to a [`MapLike`] as an optional field with a provided name.
pub trait OptionalFieldEncode {
    /// Encodes this value to a map by adding an optional field, whose:
    /// - key is the field's `name`.
    /// - value is the encoded value represented by the provided [`DynamicOps`].
    fn encode_optional_field<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        prefix: B,
        ops: &O,
        name: impl Into<String>,
    ) -> B;
}

impl<T> OptionalFieldEncode for Option<&T>
where
    T: Encode,
{
    fn encode_optional_field<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        prefix: B,
        ops: &O,
        name: impl Into<String>,
    ) -> B {
        if let Some(value) = self {
            value.encode_field(prefix, ops, name)
        } else {
            prefix
        }
    }
}

/// A trait to decode an optional field of a [`MapLike`] into a
/// value of the implementing type.
pub trait OptionalFieldDecode: Sized {
    /// Decodes an optional field from a map, similar to [`FieldDecode::decode_field`].
    ///
    /// However, this method has an extra `lenient` parameter. If it is `true`, errors
    /// while decoding a `Some` option will not occur, and a `None` will be decoded instead.
    fn decode_optional_field<O: DynamicOps>(
        input: &mut impl MapLike<Value = O::Value>,
        ops: &O,
        name: &'static str,
        lenient: bool,
    ) -> DataResult<Self>;
}

impl<T> OptionalFieldDecode for Option<T>
where
    T: Decode,
{
    fn decode_optional_field<O: DynamicOps>(
        input: &mut impl MapLike<Value = O::Value>,
        ops: &O,
        name: &'static str,
        lenient: bool,
    ) -> DataResult<Self> {
        let value = input.remove(name);
        let Some(value) = value else {
            return DataResult::success(None);
        };
        let parsed = T::decode(ops, value);
        if parsed.is_error() && lenient {
            return DataResult::success(None);
        }

        match parsed {
            DataResult {
                kind: DataResultKind::Success(t),
                lifecycle,
            } => DataResult {
                kind: DataResultKind::Success(Some(t)),
                lifecycle,
            },
            DataResult {
                kind: DataResultKind::Error { partial, error },
                lifecycle,
            } => DataResult {
                kind: DataResultKind::Error {
                    partial: partial.map(Some),
                    error,
                },
                lifecycle,
            },
        }
    }
}
