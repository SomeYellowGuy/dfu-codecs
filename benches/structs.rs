use criterion::{criterion_group, criterion_main};
use dfu_codecs::codec::{Decode, MapDecode, MapEncode};
use dfu_codecs::{
    DataResult, DynamicOps, MapLike, RecordBuilder, decode_from_map_decode, encode_from_map_encode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod common;

#[derive(Serialize, Deserialize)]
pub struct Abc {
    a: i32,
    b: String,
    c: bool,
}
fn abc() -> Abc {
    Abc {
        a: 1,
        b: "Hello world!".to_string(),
        c: true,
    }
}
fn abc_json() -> serde_json::Value {
    json!({ "a": 1, "b": "Hello world!", "c": true })
}

impl MapEncode for Abc {
    fn map_encode<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        ops: &O,
        prefix: B,
    ) -> B {
        prefix
            .add_field(ops, "a", &self.a)
            .add_field(ops, "b", &self.b)
            .add_field(ops, "c", &self.c)
    }
}
encode_from_map_encode!(Abc);
impl MapDecode for Abc {
    fn map_decode<O: DynamicOps>(
        ops: &O,
        input: &impl MapLike<Value = O::Value>,
    ) -> DataResult<Self> {
        let a = i32::decode_field(input, ops, "a");
        let b = String::decode_field(input, ops, "b");
        let c = bool::decode_field(input, ops, "c");
        DataResult::apply_3(|a, b, c| Abc { a, b, c }, a, b, c)
    }
}
decode_from_map_decode!(Abc);

#[derive(Serialize, Deserialize)]
pub struct Abcd {
    abc: Abc,
    d: f64,
}

impl MapEncode for Abcd {
    fn map_encode<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        ops: &O,
        prefix: B,
    ) -> B {
        prefix
            .add_field(ops, "abc", &self.abc)
            .add_field(ops, "d", &self.d)
    }
}
encode_from_map_encode!(Abcd);
impl MapDecode for Abcd {
    fn map_decode<O: DynamicOps>(
        ops: &O,
        input: &impl MapLike<Value = O::Value>,
    ) -> DataResult<Self> {
        let abc = Abc::decode_field(input, ops, "abc");
        let d = f64::decode_field(input, ops, "d");
        DataResult::apply_2(|abc, d| Abcd { abc, d }, abc, d)
    }
}
decode_from_map_decode!(Abcd);

#[derive(Serialize, Deserialize)]
pub struct Unit;

impl MapEncode for Unit {
    fn map_encode<O: DynamicOps, B: RecordBuilder<Value = O::Value>>(
        &self,
        _ops: &O,
        prefix: B,
    ) -> B {
        prefix
    }
}
encode_from_map_encode!(Unit);
impl MapDecode for Unit {
    fn map_decode<O: DynamicOps>(
        _ops: &O,
        _input: &impl MapLike<Value = O::Value>,
    ) -> DataResult<Self> {
        DataResult::success(Unit)
    }
}
decode_from_map_decode!(Unit);

bench_encode_and_decode_with_serde!(
    unit_benches,
    Unit,
    encode { unit_encode: Unit },
    decode {
        unit_decode: json!({})
    }
);

bench_encode_and_decode_with_serde!(
    simple_benches,
    Abc,
    encode {
        simple_encode: abc()
    },
    decode {
        simple_decode: abc_json(),
        simple_decode_no_key: json!({ "a": 1, "b": "Hello world!" }),
        simple_decode_wrong_type: json!({ "a": 1, "b": "Hello world!", "c": 4 })
    }
);

bench_encode_and_decode_with_serde!(
    composite_benches,
    Abcd,
    encode {
        composite_encode: Abcd { abc: abc(), d: 1.5 }
    },
    decode {
        composite_decode: json!({"abc": abc_json(), "d": 1.23}),
        composite_decode_no_key: json!({"abc": abc_json()}),
        composite_decode_wrong_type: json!({"abc": abc_json(), "d": true})
    }
);

criterion_main!(unit_benches, simple_benches, composite_benches);
