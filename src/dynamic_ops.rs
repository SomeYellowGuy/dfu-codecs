use crate::{DataResult};
use crate::builder::{ListBuilder, RecordBuilder};
use crate::number::Number;
use std::fmt::{Debug, Display};
use thiserror::Error;

macro_rules! impl_try_list_wrapper {
    ($name:ident | $ty:ty | $type_name:literal) => {
        fn $name(&self, input: Self::Value) -> DataResult<Vec<$ty>> {
            if self.data_type(&input) != DataType::List {
                // This check guarantees that the data type is a list, so we can
                // safely return the result directly.
                // TODO: Put unwrap error here
                return self.try_list(input).map(|_| Vec::new());
            }
            let possible_input_error =
                format!(concat!("Some elements are not ", $type_name, ": {}"), input);

            self.try_list(input).and_then(|l| {
                let mut array = Vec::new();
                for n in l {
                    let Some(n) = self.try_number(n).into_success() else {
                        return DataResult::error_string(possible_input_error);
                    };
                    array.push(<$ty>::from(n));
                }
                DataResult::success(array)
            })
        }
    };
}

#[derive(Error, Debug)]
pub enum GenericOpsError {
    #[error("Do not know how to append a primitive value {0} to {1}")]
    DoNotKnowHowToAppendPrimitive(String, String),
}

pub trait DynamicOps: Sized + 'static {
    type Value: Debug + Display + Clone;

    fn empty(&self) -> Self::Value;
    fn empty_list(&self) -> Self::Value;
    fn empty_map(&self) -> Self::Value;

    fn number(&self, value: Number) -> Self::Value;

    fn byte(&self, value: i8) -> Self::Value {
        self.number(Number::Byte(value))
    }
    fn short(&self, value: i16) -> Self::Value {
        self.number(Number::Short(value))
    }
    fn int(&self, value: i32) -> Self::Value {
        self.number(Number::Int(value))
    }
    fn long(&self, value: i64) -> Self::Value {
        self.number(Number::Long(value))
    }
    fn float(&self, value: f32) -> Self::Value {
        self.number(Number::Float(value))
    }
    fn double(&self, value: f64) -> Self::Value {
        self.number(Number::Double(value))
    }

    fn bool(&self, value: bool) -> Self::Value;
    fn string(&self, value: String) -> Self::Value;

    fn byte_buffer(&self, value: &[i8]) -> Self::Value {
        self.list(value.iter().map(|v| self.byte(*v)))
    }
    fn int_stream(&self, value: &[i32]) -> Self::Value {
        self.list(value.iter().map(|v| self.int(*v)))
    }
    fn long_stream(&self, value: &[i64]) -> Self::Value {
        self.list(value.iter().map(|v| self.long(*v)))
    }

    fn data_type(&self, value: &Self::Value) -> DataType;

    fn list(&self, value: impl IntoIterator<Item = Self::Value>) -> Self::Value;
    fn map(&self, value: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Value;

    fn try_number(&self, input: Self::Value) -> DataResult<Number>;
    fn try_bool(&self, input: Self::Value) -> DataResult<bool>;
    fn try_string(&self, input: Self::Value) -> DataResult<String>;

    impl_try_list_wrapper!(try_byte_list | i8 | "bytes");
    impl_try_list_wrapper!(try_int_list | i32 | "ints");
    impl_try_list_wrapper!(try_long_list | i64 | "longs");

    fn try_list(&self, input: Self::Value) -> DataResult<Vec<Self::Value>>;
    fn try_map(&self, input: Self::Value) -> DataResult<impl MapLike<Value = Self::Value>>;

    fn list_builder(&self) -> impl ListBuilder<Value = Self::Value>;
    fn map_builder(&self) -> impl RecordBuilder<Value = Self::Value>;

    fn merge_to_primitive(
        &self,
        prefix: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value> {
        if self.data_type(&prefix) != DataType::Empty {
            let string_value = value.to_string();
            return DataResult::partial(
                value,
                GenericOpsError::DoNotKnowHowToAppendPrimitive(string_value, prefix.to_string()),
            );
        }
        DataResult::success(value)
    }

    fn merge_to_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value>;

    fn merge_values_to_list(
        &self,
        list: Self::Value,
        values: impl IntoIterator<Item = Self::Value>,
    ) -> DataResult<Self::Value> {
        let mut result = DataResult::success(list);

        for value in values {
            result = result.and_then(|r| self.merge_to_list(r, value))
        }
        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    Empty,
    Number,
    String,
    Boolean,
    List,
    Map,
}

pub trait MapLike {
    type Value: Display;

    fn get(&self, key: &str) -> Option<&Self::Value>;

    fn remove(&mut self, key: &str) -> Option<Self::Value>;

    fn into_entries(self) -> impl Iterator<Item = (String, Self::Value)>;
}
