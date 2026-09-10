use criterion::{criterion_group, criterion_main};
use dfu_codecs::JsonOps;
use dfu_codecs::codec::{BoundedVec, Encode};
use serde_json::json;

mod common;

fn vec() -> Vec<i64> {
    (1..100).into_iter().collect()
}

fn vec_json() -> serde_json::Value {
    vec().encode_start(&JsonOps).unwrap()
}

bench_encode_and_decode_with_serde!(
    vec_benches,
    Vec<i64>,
    encode { vec_encode: vec() },
    decode {
        vec_decode: json!(vec_json()),
        vec_decode_wrong_type: json!([1000, 2000, 3000, "not a number"])
    }
);

bench_encode_and_decode!(
    bounded_vec_benches, BoundedVec<f32, 3, 5>,
    encode {
        bounded_vec_encode: BoundedVec(vec![1.45, 123.934, 0.0, -9274.0, 0.001]),
        bounded_vec_encode_too_short: BoundedVec(vec![1.45, 123.934]),
        bounded_vec_encode_too_long: BoundedVec(vec![1.45, 123.934, 0.0, -9274.0, 0.001, 99.0]),
    },
    decode {
        bounded_vec_decode: json!([1.45, 123.934, 0.0, -9274.0, 0.001]),
        bounded_vec_decode_too_short: json!([1.45, 123.934]),
        bounded_vec_decode_too_long: json!([1.45, 123.934, 0.0, -9274.0, 0.001, 99.0]),
    }
);

bench_encode_and_decode!(
    array_slice_benches,
    [i8; 5],
    encode {
        array_slice_encode: [1, 2, 3, 4, 5]
    },
    decode {
        array_slice_decode: json!([1, 2, 3, 4, 5]),
        array_slice_wrong_size: json!([1, 2, 3, 4, 5, 6])
    }
);

criterion_main!(vec_benches, bounded_vec_benches, array_slice_benches);
