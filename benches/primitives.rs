mod common;

use criterion::criterion_group;
use criterion::criterion_main;
use dfu_codecs::JsonOps;
use dfu_codecs::codec::{Encode, LongStream};
use serde_json::json;

fn create_string(len: usize) -> String {
    "a".repeat(len)
}

fn create_string_json(len: usize) -> serde_json::Value {
    create_string(len).encode_start(&JsonOps).unwrap()
}

bench_encode_and_decode_with_serde!(
    number_benches,
    i32,
    encode { number_encode: 7 },
    decode {
        number_decode: json!(7)
    }
);

bench_encode_and_decode_with_serde!(
    string_benches,
    String,
    encode {
        string_len_10_encode: create_string(10),
        string_len_1000_encode: create_string(1000),
        string_len_100000_encode: create_string(100000),
    },
    decode {
        string_len_10_decode: create_string_json(10),
        string_len_1000_decode: create_string_json(1000),
        string_len_100000_decode: create_string_json(100000)
    }
);

bench_encode_and_decode_with_serde!(
    boolean_benches,
    bool,
    encode {
        boolean_encode: true,
    },
    decode {
        boolean_decode: json!(true)
    }
);

bench_encode_and_decode!(
    list_wrapper_benches,
    LongStream,
    encode {
        list_wrapper_encode: vec![24_134, -12_349_123, 287_941_234].into(),
    },
    decode {
        list_wrapper_decode: json!([24_134, -12_349_123, 287_941_234])
    }
);

criterion_main!(
    number_benches,
    string_benches,
    boolean_benches,
    list_wrapper_benches,
);
