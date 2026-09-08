use crate::DataResult;
use crate::builder::{ListBuilder, RecordBuilder};
use crate::dynamic_ops::{DataType, DynamicOps, MapLike};
use crate::number::Number;
use serde_json::{Map, Value};
use thiserror::Error;

pub struct JsonOps;

#[derive(Error, Debug)]
pub enum JsonOpsError {
    #[error("Not a number: {0}")]
    NotNumber(Value),
    #[error("Not a boolean: {0}")]
    NotBool(Value),
    #[error("Not a string: {0}")]
    NotString(Value),
    #[error("Not a JSON array: {0}")]
    NotArray(Value),
    #[error("Not a JSON object: {0}")]
    NotObject(Value),
    #[error("mergeToList called with not a list: {0}")]
    MergeCalledWithNoList(Value),
    #[error("Cannot append a list to not a list: {0}")]
    CannotAppendListToNotList(String),
    #[error("Cannot append a map to not a map: {0}")]
    CannotAppendMapToNotMap(String),
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

    fn map(&self, value: impl IntoIterator<Item=(String, Self::Value)>) -> Self::Value {
        Value::Object(Map::from_iter(value))
    }

    fn try_number(&self, input: Self::Value) -> DataResult<Number> {
        if let Value::Number(n) = input {
            DataResult::success(n.into())
        } else {
            DataResult::error(JsonOpsError::NotNumber(input))
        }
    }

    fn try_bool(&self, input: Self::Value) -> DataResult<bool> {
        if let Value::Bool(b) = input {
            DataResult::success(b)
        } else {
            DataResult::error(JsonOpsError::NotBool(input))
        }
    }

    fn try_string(&self, input: Self::Value) -> DataResult<String> {
        if let Value::String(s) = input {
            DataResult::success(s)
        } else {
            DataResult::error(JsonOpsError::NotString(input))
        }
    }

    fn try_list(&self, input: Self::Value) -> DataResult<Vec<Self::Value>> {
        if let Value::Array(array) = input {
            DataResult::success(array)
        } else {
            DataResult::error(JsonOpsError::NotArray(input))
        }
    }

    fn try_map(&self, input: Self::Value) -> DataResult<impl MapLike<Value = Self::Value>> {
        if let Value::Object(map) = input {
            DataResult::success(map)
        } else {
            DataResult::error(JsonOpsError::NotObject(input))
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
            _ => DataResult::error(JsonOpsError::MergeCalledWithNoList(list)),
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
                        JsonOpsError::CannotAppendListToNotList(prefix_string),
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
                DataResult::partial(prefix, JsonOpsError::CannotAppendMapToNotMap(string_prefix))
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

    fn remove(&mut self, key: &str) -> Option<Self::Value> {
        self.remove(key)
    }

    fn into_entries(self) -> impl Iterator<Item = (String, Self::Value)> {
        self.into_iter().map(|(k, v)| (k, v))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use super::*;
    use crate::codec::{Decode, Encode, OptionalFieldDecode};

    #[test]
    fn test() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Test {
            a: i32,
            b: String,
            c: bool,
            d: Option<Box<Test>>,
            e: Vec<i8>
        }

        impl Encode for Test {
            fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
                ops.map_builder()
                    .add_field(ops, "a", &self.a)
                    .add_field(ops, "b", &self.b)
                    .add_field(ops, "c", &self.c)
                    .add_optional_field(ops, "d", self.d.as_deref())
                    .add_field(ops, "e", &self.e)
                    .build(prefix)
            }
        }

        impl Decode for Test {
            fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
                ops.try_map(input).and_then(|mut m| {
                    let a = i32::decode_field(&mut m, ops, "a");
                    let b = String::decode_field(&mut m, ops, "b");
                    let c = bool::decode_field(&mut m, ops, "c");
                    let d = Option::<Test>::decode_optional_field(&mut m, ops, "d", true)
                        .map(|r| r.map(Box::new));
                    let e = Vec::<i8>::decode_field(&mut m, ops, "e");
                    DataResult::apply_5(|a, b, c, d, e| Test { a, b, c, d, e }, a, b, c, d, e)
                })
            }
        }

        let test = Test {
            a: 69,
            b: "Cool".to_string(),
            c: true,
            d: Some(Box::new(Test {
                a: 30,
                b: String::new(),
                c: false,
                d: None,
                e: vec![1, 2, 3, -12, -43, 123]
            })),
            e: vec![]
        };

        profile(|| {
            test.encode_start(&JsonOps);
        });

        profile(|| {
            serde_json::to_value(&test).unwrap();
        });

        let json = json!({
            "a": "a",
            "b": "Cool",
            "c": true,
            "e": [1, 2, 3, -12, -43, 123]
        });

        profile(|| {
            Test::decode(&JsonOps, json.clone());
        });

        profile(|| {
            let test: Result<Test, _> = serde_json::from_value(json.clone());
        });
    }

    fn profile(f: impl Fn()) {
        let start = Instant::now();
        for _ in 0..1_000_000 {
            f()
        }
        let stop = Instant::now();

        println!("Time taken: {:?}", stop.duration_since(start));
    }
}
