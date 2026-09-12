use crate::DataResult;
use crate::codec::{Encode, OptionalFieldEncode};
use crate::dynamic_ops::DynamicOps;
use std::fmt::Display;

pub trait ListBuilder: Sized {
    type Value: Display;

    #[must_use]
    fn add(self, value: Self::Value) -> Self;

    #[must_use]
    fn add_result(self, value: DataResult<Self::Value>) -> Self;

    fn build(self, prefix: Option<Self::Value>) -> DataResult<Self::Value>;
}

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

pub trait RecordBuilder: Sized {
    type Value: Display;

    #[must_use]
    fn add(self, key: impl Into<String>, value: Self::Value) -> Self;

    #[must_use]
    fn add_result(self, key: impl Into<String>, value: DataResult<Self::Value>) -> Self;

    #[must_use]
    fn add_field<O: DynamicOps<Value = Self::Value>>(
        self,
        ops: &O,
        key: impl Into<String>,
        value: &impl Encode,
    ) -> Self {
        self.add_result(key, value.encode_start(ops))
    }

    #[must_use]
    fn add_optional_field<O: DynamicOps<Value = Self::Value>>(
        self,
        ops: &O,
        key: impl Into<String>,
        value: &Option<impl Encode>,
    ) -> Self {
        value.encode_optional_field(self, ops, key)
    }

    fn build(self, prefix: Option<Self::Value>) -> DataResult<Self::Value>;
}
