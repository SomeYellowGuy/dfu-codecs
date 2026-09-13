use crate::DataResult;
use crate::codec::{Encode, OptionalFieldEncode};
use crate::dynamic_ops::{DisplayValue, DynamicOps};

/// A trait providing methods to construct a list of a specific format.
pub trait ListBuilder: Sized {
    /// The final type of value this builder gives, wrapped in a [`DataResult`].
    type Value: DisplayValue;

    /// Adds the given value to this builder.
    #[must_use]
    fn add(self, value: Self::Value) -> Self;

    /// Adds the data in `value` (if any), concatenating errors if necessary.
    #[must_use]
    fn add_result(self, value: DataResult<Self::Value>) -> Self;

    /// Builds the final value, which may give an error if something went wrong
    /// while building the list.
    fn build(self, prefix: Option<Self::Value>) -> DataResult<Self::Value>;
}

/// The default implementation of a list builder.
pub struct DefaultListBuilder<'ops, O: DynamicOps> {
    ops: &'ops O,
    list: DataResult<Vec<O::Value>>,
}

impl<'ops, O: DynamicOps> DefaultListBuilder<'ops, O> {
    pub fn new(ops: &'ops O, capacity: usize) -> Self {
        Self {
            ops,
            list: DataResult::success(Vec::with_capacity(capacity)),
        }
    }
}

impl<O: DynamicOps> ListBuilder for DefaultListBuilder<'_, O> {
    type Value = O::Value;

    fn add(mut self, value: Self::Value) -> Self {
        self.list = self.list.map(|mut v| {
            v.push(value);
            v
        });
        self
    }

    fn add_result(mut self, value: DataResult<Self::Value>) -> Self {
        self.list = DataResult::apply_2_stable(
            |mut a, b| {
                a.push(b);
                a
            },
            self.list,
            value,
        );
        self
    }

    fn build(self, prefix: Option<Self::Value>) -> DataResult<Self::Value> {
        self.list
            .and_then(|v| self.ops.merge_values_to_list(prefix, v))
    }
}

/// A trait providing methods to construct a record (structure or map)
/// of a specific format.
pub trait RecordBuilder: Sized {
    type Value: DisplayValue;

    /// Adds the given value to this builder with its corresponding key.
    #[must_use]
    fn add(self, key: impl Into<String>, value: Self::Value) -> Self;

    /// Adds the data in `value` (if any) with its corresponding key, concatenating errors if necessary.
    #[must_use]
    fn add_result(self, key: impl Into<String>, value: DataResult<Self::Value>) -> Self;

    /// Encodes a value and adds the encoded data in this builder (if any) with its corresponding key, concatenating errors if necessary.
    #[must_use]
    fn add_field<O: DynamicOps<Value = Self::Value>>(
        self,
        ops: &O,
        key: impl Into<String>,
        value: &impl Encode,
    ) -> Self {
        self.add_result(key, value.encode_start(ops))
    }

    /// Encodes an [`Option`] and adds the encoded data in this builder (if any) with its corresponding key, concatenating errors if necessary.
    #[must_use]
    fn add_optional_field<O: DynamicOps<Value = Self::Value>>(
        self,
        ops: &O,
        key: impl Into<String>,
        value: &Option<impl Encode>,
    ) -> Self {
        value.encode_optional_field(self, ops, key)
    }

    /// Builds the final value, which may give an error if something went wrong
    /// while building the record.
    fn build(self, prefix: Option<Self::Value>) -> DataResult<Self::Value>;
}
