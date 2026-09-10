use crate::builder::ListBuilder;
use crate::codec::{BuiltInError, Decode, Encode};
use crate::{DataResult, DynamicOps, Lifecycle};

pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(pub Vec<T>);

type UnboundedVec<T> = BoundedVec<T, 0, { usize::MAX }>;

impl<T, const MIN: usize, const MAX: usize> BoundedVec<T, MIN, MAX> {
    fn too_short_error(size: usize) -> BuiltInError {
        BuiltInError::TooShortList {
            size,
            min_size: MIN,
            max_size: MAX,
        }
    }

    fn too_long_error(size: usize) -> BuiltInError {
        BuiltInError::TooLongList {
            size,
            min_size: MIN,
            max_size: MAX,
        }
    }
}

fn encode<T: Encode, O: DynamicOps, const MIN: usize, const MAX: usize>(
    vec: &[T],
    ops: &O,
    prefix: O::Value,
) -> DataResult<O::Value> {
    if vec.len() < MIN {
        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_short_error(vec.len()));
    }
    if vec.len() > MAX {
        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_long_error(vec.len()));
    }
    let mut builder = ops.list_builder();
    for element in vec {
        builder = builder.add_result(element.encode_start(ops));
    }
    builder.build(prefix)
}

impl<T: Encode, const MIN: usize, const MAX: usize> Encode for BoundedVec<T, MIN, MAX> {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        encode::<T, O, MIN, MAX>(&self.0, ops, prefix)
    }
}

impl<T: Decode, const MIN: usize, const MAX: usize> Decode for BoundedVec<T, MIN, MAX> {
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        ops.try_list(input)
            .with_lifecycle(Lifecycle::Stable)
            .and_then(|l| {
                // Optimization: Check for l being too short before doing anything.
                // Unlike DFU, we don't return a "remaining" value for Decode, so we don't need to collect
                // failed entries.
                if l.len() < MIN {
                    return DataResult::error(Self::too_short_error(l.len()));
                }
                let mut result = DataResult::success_with_lifecycle((), Lifecycle::Stable);
                let mut elements = Vec::with_capacity(l.len());
                let mut total_count = 0;
                for element in l {
                    total_count += 1;
                    if elements.len() >= MAX {
                        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_long_error(
                            total_count,
                        ));
                    }
                    let element_result = T::decode(ops, element);
                    result = DataResult::apply_2_stable(
                        |_, element| elements.push(element),
                        result,
                        element_result,
                    )
                }
                if elements.len() < MIN {
                    return DataResult::error(Self::too_short_error(total_count));
                }
                let decoded = BoundedVec(elements);

                result.with_data(decoded)
            })
    }
}

impl<T> From<Vec<T>> for UnboundedVec<T> {
    fn from(value: Vec<T>) -> Self {
        BoundedVec(value)
    }
}

impl<T, const MIN: usize, const MAX: usize> From<BoundedVec<T, MIN, MAX>> for Vec<T> {
    fn from(value: BoundedVec<T, MIN, MAX>) -> Self {
        value.0
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        encode::<T, O, 0, { usize::MAX }>(self, ops, prefix)
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        UnboundedVec::decode(ops, input).map(|vec| vec.0)
    }
}

impl<T: Clone + Encode, const N: usize> Encode for [T; N] {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        self.to_vec().encode(ops, prefix)
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        <BoundedVec<T, N, N>>::decode(ops, input)
            .map(|vec| <[T; N]>::try_from(vec.0).unwrap_or_else(|_| unreachable!()))
    }
}

#[cfg(test)]
mod tests {
    use crate::json_ops::JsonOps;
    use crate::{assert_decode_error, assert_decode_success, assert_encode_success};
    use serde_json::json;

    #[test]
    fn encoding() {
        assert_encode_success!(JsonOps, vec![1, 2] => json!([1, 2]));

        assert_encode_success!(JsonOps, Vec::<i32>::new() => json!([]));
        assert_encode_success!(JsonOps, vec![-3, 192, 182] => json!([-3, 192, 182]));
        assert_encode_success!(
            JsonOps,
            vec!["a".to_string(), "b".to_string()] => json!(["a", "b"])
        );
        assert_encode_success!(JsonOps, vec!["one".to_string()] => json!(["one"]));
        assert_encode_success!(
            JsonOps,
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
                => json!(["1", "2", "3"])
        );
        assert_encode_success!(JsonOps, vec![1, 2] => json!([1, 2]));
        assert_encode_success!(JsonOps, vec![true, false] => json!([true, false]));
        assert_encode_success!(
            JsonOps,
            vec![vec![true, false], vec![true, false]]
                => json!([[true, false], [true, false]])
        );
        assert_encode_success!(
            JsonOps,
            vec![vec![vec![true, true], vec![false, false]]]
                => json!([[[true, true], [false, false]]])
        );
    }

    #[test]
    fn decoding() {
        type NumberGrid = Vec<Vec<f64>>;

        assert_decode_success!(JsonOps, Vec<i16>, json!([-1, -2, -3]) => vec![-1, -2, -3]);
        assert_decode_success!(JsonOps, Vec<i16>, json!([1, 2, 6, 24, 120]) => vec![1, 2, 6, 24, 120]);
        assert_decode_error!(JsonOps, Vec<i16>, json!(["string", "b"]) => "Not a number: \"string\"; Not a number: \"b\"");
        assert_decode_error!(JsonOps, Vec<i16>, json!(false) => "Not a JSON array: false");

        assert_decode_success!(JsonOps, NumberGrid, json!([[0, 0.5, 1.0]]) => vec![vec![0.0, 0.5, 1.0]]);
        assert_decode_success!(
            JsonOps,
            NumberGrid,
            json!([[0, 0.5, 1.0], [1, 4, 5], [-293.4, 1, 293]]) => vec![vec![0.0, 0.5, 1.0], vec![1.0, 4.0, 5.0], vec![-293.4, 1.0, 293.0]]
        );
        assert_decode_error!(
            JsonOps,
            NumberGrid,
            json!([[0, 0.5, 1.0], [1, false, 5], [-293.4, 1, 293]]) => "Not a number: false"
        );
        assert_decode_error!(
            JsonOps,
            NumberGrid,
            json!([[0, 0.5, 1.0], [1, false, 5], [false, 1, "hi"]]) => "Not a number: false; Not a number: false; Not a number: \"hi\""
        );
        assert_decode_success!(
            JsonOps,
            NumberGrid,
            json!([[1, 1.5, 2.0], [-20]]) => vec![vec![1.0, 1.5, 2.0], vec![-20.0]]
        );
        assert_decode_success!(JsonOps, NumberGrid, json!([[]]) => vec![Vec::<f64>::new()]);
        assert_decode_error!(JsonOps, NumberGrid, json!([[[[]]]]) => "Not a number: [[]]");
    }
}
