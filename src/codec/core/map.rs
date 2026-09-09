use std::collections::hash_map::Entry;
use crate::codec::{BuiltInError, Decode, Encode};
use crate::data_result::{DataResult, ErrorMessage};
use crate::{DynamicOps, Lifecycle, MapLike, RecordBuilder};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

fn base_map_encode<O, K, V, B>(mut prefix: B, ops: &O, input: &HashMap<K, V>) -> B
where
    O: DynamicOps,
    for<'a> &'a K: Into<String>,
    K: Clone,
    V: Encode,
    B: RecordBuilder<Value = O::Value>,
{
    for (key, value) in input {
        prefix = prefix.add_result(key, value.encode_start(ops))
    }
    prefix
}

fn base_map_decode<O, K, V>(
    ops: &O,
    input: &impl MapLike<Value = O::Value>,
) -> DataResult<HashMap<K, V>>
where
    O: DynamicOps,
    O::Value: Clone,
    K: Decode + Eq + Hash + Display,
    V: Decode,
{
    let mut elements = HashMap::new();
    let mut failed = Vec::new();

    let result = input.entries().fold(
        DataResult::success_with_lifecycle((), Lifecycle::Stable),
        |r, (key, value)| {
            let key_result = K::decode(ops, &ops.string(key.to_owned()));
            let value_result = V::decode(ops, value);

            let pair = DataResult::apply_2_stable(|k, v| (k, v), key_result, value_result);
            let is_error = pair.is_error();
            let (pair_option, rest) = pair.extract();

            if let Some((decoded_key, decoded_value)) = pair_option {
                match elements.entry(decoded_key) {
                    Entry::Occupied(entry) => {
                        failed.push((key, value));
                        return DataResult::apply_2_stable::<(), ()>(
                            |u, _| u,
                            r,
                            DataResult::error(BuiltInError::DuplicateEntry(
                                entry.key().to_string().into(),
                            )),
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(decoded_value);
                    }
                }
            }
            if is_error {
                failed.push((key, value));
            }

            DataResult::apply_2_stable(|u, _| u, r, rest)
        },
    );

    let errors = ops.map(failed.into_iter().map(|(k, v)| (k.to_owned(), v.clone())));
    result
        .with_data(elements)
        .add_message_if_error(|| ErrorMessage::additional(format!(" missed input: {errors}")))
}

impl<K, V> Encode for HashMap<K, V>
where
    K: Encode + Clone + Hash,
    for<'a> &'a K: Into<String>,
    V: Encode,
{
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        base_map_encode(ops.map_builder(), ops, self).build(prefix)
    }
}

impl<K, V> Decode for HashMap<K, V>
where
    K: Decode + Eq + Hash + Display,
    V: Decode,
{
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        ops.try_map(input)
            .with_lifecycle(Lifecycle::Stable)
            .and_then(|map| base_map_decode(ops, map))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DataResult, DataTryFrom, JsonOps, assert_decode_error, assert_decode_success,
        assert_encode_success, comap_flat_map_codec_impl,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};

    #[test]
    fn simple_encoding() {
        let mut map = HashMap::<String, i32>::new();

        map.insert("Red".to_string(), 21);
        map.insert("Yellow".to_string(), 51);
        map.insert("Blue".to_string(), 98);
        map.insert("Green".to_string(), 5);

        assert_encode_success!(JsonOps, map => json!({"Red": 21, "Yellow": 51, "Blue": 98, "Green": 5}));
    }

    #[test]
    fn string_integer_map() {
        // A basic implementation to check if a number is prime.
        fn is_prime(number: u32) -> bool {
            if number < 2 {
                return false;
            }
            for i in 2..number {
                if number.is_multiple_of(i) {
                    return false;
                }
            }
            true
        }

        /// A `u32` wrapper that encodes a `String`.
        #[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
        struct StringInteger(u32);

        type StringIntegerMap = HashMap<StringInteger, bool>;

        impl Display for StringInteger {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&StringInteger> for String {
            fn from(value: &StringInteger) -> Self {
                // This will always succeed.
                value.0.to_string()
            }
        }

        impl DataTryFrom<String> for StringInteger {
            fn data_try_from(value: String) -> DataResult<Self> {
                // Try to parse an integer.
                value.parse().map_or_else(
                    |_| DataResult::error("Could not parse string"),
                    |i| DataResult::success(Self(i)),
                )
            }
        }

        comap_flat_map_codec_impl!(String => StringInteger, StringInteger::data_try_from, String::from);

        let mut map = StringIntegerMap::new();

        // Calculate the map for the first 20 numbers.
        for i in 1..=20 {
            map.insert(StringInteger(i), is_prime(i));
        }

        assert_encode_success!(
            JsonOps,
            map =>
            json!({
                "1": false, "2": true, "3": true, "4": false, "5": true, "6": false, "7": true, "8": false, "9": false, "10": false,
                "11": true, "12": false, "13": true, "14": false, "15": false, "16": false, "17": true, "18": false, "19": true, "20": false
            })
        );

        assert_decode_success!(
            JsonOps,
            StringIntegerMap,
            json!({
                "1": true, "2": true, "3": true, "4": false, "5": false
            })
        );
        assert_decode_success!(JsonOps, StringIntegerMap, json!({}));
        assert_decode_error!(
            JsonOps,
            StringIntegerMap,
            json!("Definitely a map") => "Not a JSON object: \"Definitely a map\"",
        );

        assert_decode_error!(
            JsonOps,
            StringIntegerMap,
            json!({
                "1": true, "5": 3
            }) => "Not a boolean: 3 missed input: {\"5\":3}"
        );

        assert_decode_error!(
            JsonOps,
            StringIntegerMap,
            json!({
                "0": true, "5": -99, "89": [1, 2, 3]
            }) => "Not a boolean: -99; Not a boolean: [1,2,3] missed input: {\"5\":-99,\"89\":[1,2,3]}"
        );
    }
}
