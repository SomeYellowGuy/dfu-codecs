mod common;
use serde_json::json;
use criterion::criterion_main;
use criterion::criterion_group;

common_bench_functions!();

const TEST_STR: &str = "ABCDEabcde123#$";

bench_encode_and_decode!(
    number_benches | i32,
    encode {
        number_encode: 7
    },
    decode {
        number_decode: json!(7)
    }
);

bench_encode_and_decode!(
    string_benches | String,
    encode {
        string_encode: TEST_STR.to_string(),
    },
    decode {
        string_decode: json!(TEST_STR)
    }
);

bench_encode_and_decode!(
    boolean_benches | bool,
    encode {
        boolean_encode: true,
    },
    decode {
        boolean_decode: json!(1)
    }
);

criterion_main!(
    number_benches,
    string_benches,
    boolean_benches,
);