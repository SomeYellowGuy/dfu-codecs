use crate::builder::ListBuilder;
use crate::codec::{BuiltInError, Decode, Encode};
use crate::{DataResult, DynamicOps, Lifecycle};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(pub Vec<T>);

type UnboundedVec<T> = BoundedVec<T, 0, { usize::MAX }>;

impl<T, const MIN: usize, const MAX: usize> BoundedVec<T, MIN, MAX> {
    const fn too_short_error(size: usize) -> BuiltInError {
        BuiltInError::TooShortList {
            size,
            min_size: MIN,
            max_size: MAX,
        }
    }

    const fn too_long_error(size: usize) -> BuiltInError {
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
    prefix: Option<O::Value>,
) -> DataResult<O::Value> {
    if vec.len() < MIN {
        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_short_error(vec.len()));
    }
    if vec.len() > MAX {
        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_long_error(vec.len()));
    }
    let mut builder = ops.list_builder(vec.len());
    for element in vec {
        builder = builder.add_result(element.encode_start(ops));
    }
    builder.build(prefix)
}

impl<T: Encode, const MIN: usize, const MAX: usize> Encode for BoundedVec<T, MIN, MAX> {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: Option<O::Value>) -> DataResult<O::Value> {
        encode::<T, O, MIN, MAX>(&self.0, ops, prefix)
    }
}

impl<T: Decode, const MIN: usize, const MAX: usize> Decode for BoundedVec<T, MIN, MAX> {
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        ops.try_list(input)
            .with_lifecycle(Lifecycle::Stable)
            .and_then(|l| {
                // Optimization: Check for l being too short before doing anything.
                if l.len() < MIN {
                    return DataResult::error(Self::too_short_error(l.len()));
                }
                let mut result = DataResult::success_with_lifecycle((), Lifecycle::Stable);
                let mut elements = Vec::with_capacity(l.len());
                let mut total_count = 0;
                for element in l {
                    total_count += 1;
                    if elements.len() >= MAX {
                        result = DataResult::error(BoundedVec::<T, MIN, MAX>::too_long_error(
                            total_count,
                        ));
                        break;
                    }
                    let element_result = T::decode(ops, element);
                    let (partial, rest) = element_result.extract();
                    if let Some(value) = partial {
                        elements.push(value);
                    }
                    result = DataResult::apply_2_stable(|(), ()| (), result, rest);
                }
                if elements.len() < MIN {
                    return DataResult::error(Self::too_short_error(elements.len()));
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
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: Option<O::Value>) -> DataResult<O::Value> {
        encode::<T, O, 0, { usize::MAX }>(self, ops, prefix)
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        UnboundedVec::decode(ops, input).map(|vec| vec.0)
    }
}

impl<T: Clone + Encode, const N: usize> Encode for [T; N] {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: Option<O::Value>) -> DataResult<O::Value> {
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
    use crate::codec::BoundedVec;
    use crate::json_ops::JsonOps;
    use crate::{
        assert_decode_error, assert_decode_success, assert_encode_error, assert_encode_success,
    };
    use serde_json::json;

    #[test]
    fn unbounded_encoding() {
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
    fn unbounded_decoding() {
        type NumberGrid = Vec<Vec<f64>>;

        assert_decode_success!(JsonOps, Vec<i16>, json!([-1, -2, -3]) => vec![-1, -2, -3]);
        assert_decode_success!(JsonOps, Vec<i16>, json!([1, 2, 6, 24, 120]) => vec![1, 2, 6, 24, 120]);
        assert_decode_error!(JsonOps, Vec<i16>, json!(["string", "b"]) => "Not a number: \"b\"; Not a number: \"string\"");
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
            json!([[0, 0.5, 1.0], [1, false, 5], [false, 1, "hi"]]) => "Not a number: \"hi\"; Not a number: false; Not a number: false"
        );
        assert_decode_success!(
            JsonOps,
            NumberGrid,
            json!([[1, 1.5, 2.0], [-20]]) => vec![vec![1.0, 1.5, 2.0], vec![-20.0]]
        );
        assert_decode_success!(JsonOps, NumberGrid, json!([[]]) => vec![Vec::<f64>::new()]);
        assert_decode_error!(JsonOps, NumberGrid, json!([[[[]]]]) => "Not a number: [[]]");
    }

    #[test]
    fn bounded_encoding() {
        assert_encode_success!(JsonOps, BoundedVec::<_, 1, 3>(vec![1, 2, 3]) => json!([1, 2, 3]));
        assert_encode_success!(JsonOps, BoundedVec::<_, 2, 4>(vec![1, 2, 3]) => json!([1, 2, 3]));
        assert_encode_success!(JsonOps, BoundedVec::<_, 2, 4>(vec![1, 2, 3]) => json!([1, 2, 3]));

        assert_encode_error!(JsonOps, BoundedVec::<_, 2, 2>(vec![1, 2, 3]) => "List is too long: 3, expected range [2-2]");
        assert_encode_error!(JsonOps, BoundedVec::<_, 2, 2>(vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]) => "List is too long: 3, expected range [2-2]");
    }

    #[test]
    fn bounded_decoding() {
        assert_decode_success!(JsonOps, BoundedVec<i32, 1, 3>, json!([1, 2, 3]) => BoundedVec(vec![1, 2, 3]));
        assert_decode_success!(JsonOps, BoundedVec<i32, 4, 4>, json!([1, 2.0, 3.0, 4]) => BoundedVec(vec![1, 2, 3, 4]));
        assert_decode_success!(JsonOps, BoundedVec<BoundedVec<bool, 1, 3>, 1, 3>, json!([[true, false, false], [false], [false]]) => BoundedVec(vec![BoundedVec(vec![true, false, false]), BoundedVec(vec![false]), BoundedVec(vec![false])]));

        assert_decode_error!(JsonOps, BoundedVec<bool, 0, 2>, json!([true, false, true]) => "List is too long: 3, expected range [0-2]");
        assert_decode_error!(JsonOps, BoundedVec<BoundedVec<bool, 1, 3>, 1, 3>, json!([[true, false, true, false], [], [false]]) => "List is too short: 0, expected range [1-3]; List is too long: 4, expected range [1-3]");
    }
}
