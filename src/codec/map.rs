use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use crate::{DataResult, DynamicOps, Lifecycle, MapLike, RecordBuilder};
use crate::codec::{Decode, Encode};
use crate::data_result::ErrorMessage;

fn base_map_encode<O, K, V, B>(mut prefix: B, ops: &O, input: &HashMap<K, V>) -> B
where O: DynamicOps, K: Into<String> + Clone, V: Encode, B: RecordBuilder<Value = O::Value> {
    for (key, value) in input {
        prefix = prefix.add_result(
            key.clone(), value.encode_start(ops),
        )
    }
    prefix
}


fn base_map_decode<O, K, V>(ops: &O, input: impl MapLike<Value = O::Value>) -> DataResult<HashMap<K, V>>
where O: DynamicOps, O::Value: Clone, K: Decode + Eq + Hash + Display, V: Decode {
    let mut elements = HashMap::new();
    let mut failed = Vec::new();

    let result = input.into_entries().fold(
        DataResult::success_with_lifecycle((), Lifecycle::Stable),
        |r, pair| {
            let cloned = pair.clone();

            let key = K::decode(ops, ops.string(pair.0));
            let value = V::decode(ops, pair.1);

            let pair = DataResult::apply_2_stable(|k, v| (k, v), key, value);
            let is_error = pair.is_error();
            let (pair_option, rest) = pair.extract();

            if let Some((key, value)) = pair_option {
                if elements.contains_key(&key) {
                    failed.push(cloned);
                    return DataResult::apply_2_stable::<(), ()>(|u, _| u, r, DataResult::error_string(format!("Duplicate entry for key: '{key}'")))
                } else {
                    elements.insert(key, value);
                }
            }
            if is_error {
                failed.push(cloned);
            }

            DataResult::apply_2_stable(|u, _| u, r, rest)
        }
    );

    let errors = ops.map(failed);
    result.with_data(elements)
        .add_message_if_error(|| ErrorMessage::additional(format!(" missed input: {errors}")))
}

impl<K, V> Encode for HashMap<K, V>
    where K: Encode + Into<String> + Clone + Hash, V: Encode {

    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        base_map_encode(ops.map_builder(), ops, &self).build(prefix)
    }
}

impl<K, V> Decode for HashMap<K, V>
where K: Decode + Eq + Hash + Display, V: Decode {
    fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        ops.try_map(input)
            .with_lifecycle(Lifecycle::Stable)
            .and_then(|map| base_map_decode(ops, map))
    }
}