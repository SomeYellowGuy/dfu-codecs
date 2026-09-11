use crate::builder::{ListBuilder, RecordBuilder};
use crate::codec::BuiltInError;
use crate::number::Number;
use crate::{DataResult, DefaultListBuilder};
use std::fmt::{Debug, Display};

macro_rules! impl_try_list_wrapper {
    ($name:ident | $ty:ty | $type_name:literal) => {
        fn $name(&self, input: &Self::Value) -> DataResult<Vec<$ty>> {
            if self.data_type(input) != DataType::List {
                // This check guarantees that the data type is a list, so we can
                // safely return the result directly.
                return self.try_list(input).map(|_| Vec::new());
            }

            self.try_list(input).and_then(|l| {
                let mut array = Vec::with_capacity(l.len());
                for n in l {
                    let Some(n) = self.try_number(&n).into_success() else {
                        return DataResult::error(BuiltInError::SomeElementsAreDifferent(
                            $type_name,
                            input.to_string().into(),
                        ));
                    };
                    array.push(<$ty>::from(n));
                }
                DataResult::success(array)
            })
        }
    };
}

/// A trait describing methods to read and write a specific format (like NBT or JSON).
/// The `Value` of this trait is the type that can be used to represent anything in this format.
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
        self.list(value.iter().map(|&v| self.byte(v)))
    }
    fn int_stream(&self, value: &[i32]) -> Self::Value {
        self.list(value.iter().map(|&v| self.int(v)))
    }
    fn long_stream(&self, value: &[i64]) -> Self::Value {
        self.list(value.iter().map(|&v| self.long(v)))
    }

    fn data_type(&self, value: &Self::Value) -> DataType;

    fn list(&self, value: impl IntoIterator<Item = Self::Value>) -> Self::Value;
    fn map(&self, value: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Value;

    fn try_number(&self, input: &Self::Value) -> DataResult<Number>;

    fn try_byte(&self, input: &Self::Value) -> DataResult<i8> {
        self.try_number(input).map(i8::from)
    }
    fn try_short(&self, input: &Self::Value) -> DataResult<i16> {
        self.try_number(input).map(i16::from)
    }
    fn try_int(&self, input: &Self::Value) -> DataResult<i32> {
        self.try_number(input).map(i32::from)
    }
    fn try_long(&self, input: &Self::Value) -> DataResult<i64> {
        self.try_number(input).map(i64::from)
    }
    fn try_float(&self, input: &Self::Value) -> DataResult<f32> {
        self.try_number(input).map(f32::from)
    }
    fn try_double(&self, input: &Self::Value) -> DataResult<f64> {
        self.try_number(input).map(f64::from)
    }

    fn try_bool(&self, input: &Self::Value) -> DataResult<bool>;
    fn try_string(&self, input: &Self::Value) -> DataResult<String>;

    impl_try_list_wrapper!(try_byte_list | i8 | "bytes");
    impl_try_list_wrapper!(try_int_list | i32 | "ints");
    impl_try_list_wrapper!(try_long_list | i64 | "longs");

    fn try_list<'a>(&self, input: &'a Self::Value) -> DataResult<&'a [Self::Value]>;
    fn try_map<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<&'a impl MapLike<Value = Self::Value>>;

    fn list_builder(&self, capacity: usize) -> impl ListBuilder<Value = Self::Value> {
        DefaultListBuilder::new(self, capacity)
    }
    fn map_builder(&self) -> impl RecordBuilder<Value = Self::Value>;

    #[inline]
    fn merge_to_primitive(
        &self,
        prefix: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value> {
        if self.data_type(&prefix) != DataType::Empty {
            let string_value = value.to_string().into();
            return DataResult::partial(
                value,
                BuiltInError::DoNotKnowHowToAppendPrimitive(
                    string_value,
                    prefix.to_string().into(),
                ),
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

/// Provides common methods to read a specific entry or all entries of a map.
pub trait MapLike {
    type Value: Display;

    fn get(&self, key: &str) -> Option<&Self::Value>;

    fn entries(&self) -> impl Iterator<Item = (&str, &Self::Value)>;
}
