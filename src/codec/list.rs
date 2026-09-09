use crate::builder::ListBuilder;
use crate::codec::{BuiltInError, Decode, Encode};
use crate::{DataResult, DynamicOps, Lifecycle};

pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(Vec<T>);

type UnboundedVec<T> = BoundedVec<T, 0, { usize::MAX }>;

impl<T, const MIN: usize, const MAX: usize> BoundedVec<T, MIN, MAX> {
    fn too_short_error(vec: &[T]) -> BuiltInError {
        BuiltInError::TooShortList {
            size: vec.len(),
            min_size: MIN,
            max_size: MAX,
        }
    }

    fn too_long_error(vec: &[T]) -> BuiltInError {
        BuiltInError::TooLongList {
            size: vec.len(),
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
        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_short_error(vec));
    }
    if vec.len() > MAX {
        return DataResult::error(BoundedVec::<T, MIN, MAX>::too_long_error(vec));
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
    fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        ops.try_list(input)
            .with_lifecycle(Lifecycle::Stable)
            .and_then(|l| {
                let mut result = DataResult::success_with_lifecycle((), Lifecycle::Stable);
                let mut elements = Vec::new();
                for element in l {
                    if elements.len() >= MAX {
                        break;
                    }
                    let element_result = T::decode(ops, element);
                    result = DataResult::apply_2_stable(
                        |_, element| elements.push(element),
                        result,
                        element_result,
                    )
                }
                if elements.len() < MIN {
                    return DataResult::error(Self::too_short_error(&elements));
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
        encode::<T, O, 0, { usize::MAX }>(&self, ops, prefix)
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        UnboundedVec::decode(ops, input).map(|vec| vec.0)
    }
}
