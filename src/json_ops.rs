use crate::DataResult;
use crate::builder::{ListBuilder, RecordBuilder};
use crate::codec::BuiltInError;
use crate::data_result::ErrorMessage;
use crate::dynamic_ops::{DataType, DynamicOps, MapLike};
use crate::number::Number;
use serde_json::{Map, Value};
use thiserror::Error;

pub struct JsonOps;

#[derive(Error, Debug)]
pub enum JsonOpsError {
    #[error("Not a number: {0}")]
    NotNumber(Box<str>),
    #[error("Not a boolean: {0}")]
    NotBool(Box<str>),
    #[error("Not a string: {0}")]
    NotString(Box<str>),
    #[error("Not a JSON array: {0}")]
    NotArray(Box<str>),
    #[error("Not a JSON object: {0}")]
    NotObject(Box<str>),
    #[error("mergeToList called with not a list: {0}")]
    MergeCalledWithNoList(Box<str>),
    #[error("Cannot append a list to not a list: {0}")]
    CannotAppendListToNotList(Box<str>),
    #[error("Cannot append a map to not a map: {0}")]
    CannotAppendMapToNotMap(Box<str>),
}

impl From<JsonOpsError> for ErrorMessage {
    fn from(value: JsonOpsError) -> Self {
        ErrorMessage::BuiltIn(BuiltInError::Json(value))
    }
}

impl DynamicOps for JsonOps {
    type Value = Value;

    fn empty(&self) -> Self::Value {
        Value::Null
    }

    fn empty_list(&self) -> Self::Value {
        Value::Array(Vec::new())
    }

    fn empty_map(&self) -> Self::Value {
        Value::Object(Map::new())
    }

    fn number(&self, value: Number) -> Self::Value {
        value.into()
    }

    fn bool(&self, value: bool) -> Self::Value {
        Value::Bool(value)
    }

    fn string(&self, value: String) -> Self::Value {
        Value::String(value)
    }

    fn data_type(&self, value: &Self::Value) -> DataType {
        match value {
            Value::Null => DataType::Empty,
            Value::Number(_) => DataType::Number,
            Value::String(_) => DataType::String,
            Value::Bool(_) => DataType::Boolean,
            Value::Array(_) => DataType::List,
            Value::Object(_) => DataType::Map,
        }
    }

    fn list(&self, value: impl IntoIterator<Item = Value>) -> Self::Value {
        Value::Array(value.into_iter().collect())
    }

    fn map(&self, value: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Value {
        Value::Object(Map::from_iter(value))
    }

    fn try_number(&self, input: &Self::Value) -> DataResult<Number> {
        if let Value::Number(n) = input {
            DataResult::success(n.clone().into())
        } else {
            DataResult::error(JsonOpsError::NotNumber(input.to_string().into()))
        }
    }

    fn try_bool(&self, input: &Self::Value) -> DataResult<bool> {
        if let Value::Bool(b) = input {
            DataResult::success(*b)
        } else {
            DataResult::error(JsonOpsError::NotBool(input.to_string().into()))
        }
    }

    fn try_string(&self, input: &Self::Value) -> DataResult<String> {
        if let Value::String(s) = input {
            DataResult::success(s.clone())
        } else {
            DataResult::error(JsonOpsError::NotString(input.to_string().into()))
        }
    }

    fn try_list<'a>(&self, input: &'a Self::Value) -> DataResult<&'a [Value]> {
        if let Value::Array(array) = input {
            DataResult::success(array)
        } else {
            DataResult::error(JsonOpsError::NotArray(input.to_string().into()))
        }
    }

    fn try_map<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<&'a impl MapLike<Value = Self::Value>> {
        if let Value::Object(map) = input {
            DataResult::success(map)
        } else {
            DataResult::error(JsonOpsError::NotObject(input.to_string().into()))
        }
    }

    fn list_builder(&self) -> impl ListBuilder<Value = Value> {
        ArrayBuilder::new()
    }

    fn map_builder(&self) -> impl RecordBuilder<Value = Value> {
        ObjectBuilder::new()
    }

    fn merge_to_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value> {
        match list {
            Value::Null => DataResult::success(Value::Array(vec![value])),
            Value::Array(mut vec) => {
                vec.push(value);
                DataResult::success(Value::Array(vec))
            }
            _ => DataResult::error(JsonOpsError::MergeCalledWithNoList(list.to_string().into())),
        }
    }
}

pub struct ArrayBuilder(DataResult<Vec<Value>>);

impl ArrayBuilder {
    pub fn new() -> Self {
        Self(DataResult::success(Vec::new()))
    }
}

impl ListBuilder for ArrayBuilder {
    type Value = Value;

    fn add(mut self, value: Self::Value) -> Self {
        self.0 = self.0.map(|mut v| {
            v.push(value);
            v
        });
        self
    }

    fn add_result(mut self, value: DataResult<Self::Value>) -> Self {
        self.0 = DataResult::apply_2_stable(
            |mut a, b| {
                a.push(b);
                a
            },
            self.0,
            value,
        );
        self
    }

    fn build(self, prefix: Self::Value) -> DataResult<Self::Value> {
        self.0.and_then(|v| {
            let built = match prefix {
                Value::Null => v,
                Value::Array(mut prefix_vec) => {
                    prefix_vec.extend(v);
                    prefix_vec
                }
                _ => {
                    let prefix_string = prefix.to_string();
                    return DataResult::partial(
                        prefix,
                        JsonOpsError::CannotAppendListToNotList(prefix_string.to_string().into()),
                    );
                }
            };
            DataResult::success(Value::Array(built))
        })
    }
}

pub struct ObjectBuilder(DataResult<Map<String, Value>>);

impl ObjectBuilder {
    pub fn new() -> Self {
        Self(DataResult::success(Map::new()))
    }

    fn append(
        mut builder: Map<String, Value>,
        key: impl Into<String>,
        value: Value,
    ) -> Map<String, Value> {
        builder.insert(key.into(), value);
        builder
    }

    fn final_build(builder: Map<String, Value>, prefix: Value) -> DataResult<Value> {
        match prefix {
            Value::Null => DataResult::success(Value::Object(builder)),
            Value::Object(mut map) => {
                map.extend(builder);
                DataResult::success(Value::Object(map))
            }
            _ => {
                let string_prefix = prefix.to_string();
                DataResult::partial(
                    prefix,
                    JsonOpsError::CannotAppendMapToNotMap(string_prefix.to_string().into()),
                )
            }
        }
    }
}

impl RecordBuilder for ObjectBuilder {
    type Value = Value;

    fn add(mut self, key: impl Into<String>, value: Self::Value) -> Self {
        self.0 = self.0.map(|b| Self::append(b, key, value));
        self
    }

    fn add_result(mut self, key: impl Into<String>, value: DataResult<Self::Value>) -> Self {
        self.0 = DataResult::apply_2_stable(|b, v| Self::append(b, key, v), self.0, value);
        self
    }

    fn build(self, prefix: Self::Value) -> DataResult<Self::Value> {
        self.0.and_then(|b| Self::final_build(b, prefix))
    }
}

impl MapLike for Map<String, Value> {
    type Value = Value;

    fn get(&self, key: &str) -> Option<&Self::Value> {
        self.get(key)
    }

    fn entries(&self) -> impl Iterator<Item = (&str, &Self::Value)> {
        self.into_iter().map(|(k, v)| (k.as_str(), v))
    }
}
