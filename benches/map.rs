use criterion::{criterion_group, criterion_main};
use dfu_codecs::xmap_codec_impl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

mod common;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
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

xmap_codec_impl!(String => LowercaseString, LowercaseString::from, String::from);

fn hash_map() -> HashMap<LowercaseString, bool> {
    let mut map = HashMap::new();
    map.insert(LowercaseString::new("Apple"), true);
    map.insert(LowercaseString::new("Banana"), true);
    map.insert(LowercaseString::new("Orange"), false);
    map.insert(LowercaseString::new("Guava"), true);
    map.insert(LowercaseString::new("Pineapple"), false);
    map
}

bench_encode_and_decode_with_serde!(
    benches, HashMap<LowercaseString, bool>,
    encode {
        encode: hash_map(),
    },
    decode {
        decode: json!({ "Apple": true, "Banana": true, "Orange": false, "Guava": true, "Pineapple": false }),
        decode_wrong_type: json!({ "Apple": true, "Banana": true, "Orange": false, "Guava": true, "Pineapple": [false] }),
        decode_duplicate_key: json!({ "Apple": true, "Banana": true, "Orange": false, "Guava": true, "guava": false })
    }
);

criterion_main!(benches);
