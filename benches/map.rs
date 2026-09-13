use criterion::{criterion_group, criterion_main};
use dfu_codecs::xmap_codec_impl;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

mod common;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LowercaseString(String);

impl Display for LowercaseString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl LowercaseString {
    pub fn new(string: &str) -> LowercaseString {
        LowercaseString(string.to_lowercase())
    }
}

impl From<String> for LowercaseString {
    fn from(value: String) -> Self {
        LowercaseString(value.to_lowercase())
    }
}

impl From<&LowercaseString> for String {
    fn from(value: &LowercaseString) -> Self {
        value.0.to_lowercase()
    }
}

impl From<&str> for LowercaseString {
    fn from(value: &str) -> Self {
        LowercaseString::new(value)
    }
}

impl From<Cow<'_, str>> for LowercaseString {
    fn from(value: Cow<'_, str>) -> Self {
        Self::from(value.as_ref())
    }
}

xmap_codec_impl!(String => LowercaseString, LowercaseString::from, String::from);

fn hash_map() -> HashMap<String, bool> {
    let mut map = HashMap::new();
    map.insert("Apple".into(), true);
    map.insert("Banana".into(), true);
    map.insert("Orange".into(), false);
    map.insert("Guava".into(), true);
    map.insert("Pineapple".into(), false);
    map
}

bench_encode_and_decode_with_serde!(
    common_benches, HashMap<String, bool>,
    encode {
        encode: hash_map(),
    },
    decode {
        decode: json!({ "Apple": true, "Banana": true, "Orange": false, "Guava": true, "Pineapple": false }),
        decode_wrong_type: json!({ "Apple": true, "Banana": true, "Orange": false, "Guava": true, "Pineapple": [false] })
    }
);

bench_encode_and_decode!(
    specific_benches, HashMap<LowercaseString, bool>,
    encode { },
    decode {
        decode_duplicate_key: json!({ "Apple": true, "Banana": true, "Orange": false, "Guava": true, "guava": false })
    }
);

criterion_main!(common_benches, specific_benches);
