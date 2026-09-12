mod common;

use criterion::criterion_group;
use criterion::criterion_main;
use dfu_codecs::codec::LongStream;
use serde_json::json;

const TEST_STR: &str = "ABCDEabcde123#$";

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
        string_encode: TEST_STR.to_string(),
    },
    decode {
        string_decode: json!(TEST_STR)
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
