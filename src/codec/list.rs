use std::error::Error;
use crate::builder::ListBuilder;
use crate::codec::{Decode, Encode};
use crate::{DataResult, DynamicOps, Lifecycle};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
struct ListCodecError {
    kind: ListCodecErrorKind,
    size: usize,
    min_size: usize,
    max_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListCodecErrorKind {
    TooShort,
    TooLong,
}

impl Display for ListCodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ListCodecError {
                kind: ListCodecErrorKind::TooShort,
                size,
                min_size,
                max_size,
            } => write!(
                f,
                "List is too short: {size}, expected range [{min_size}-{max_size}]"
            ),
            ListCodecError {
                kind: ListCodecErrorKind::TooLong,
                size,
                min_size,
                max_size,
            } => write!(
                f,
                "List is too long: {size}, expected range [{min_size}-{max_size}]"
            ),
        }
    }
}

impl Error for ListCodecError {}

pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(Vec<T>);

type UnboundedVec<T> = BoundedVec<T, 0, { usize::MAX }>;

impl<T, const MIN: usize, const MAX: usize> BoundedVec<T, MIN, MAX> {
    fn error(vec: &[T], kind: ListCodecErrorKind) -> ListCodecError {
        ListCodecError {
            kind,
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
        return DataResult::error(BoundedVec::<T, MIN, MAX>::error(
            vec,
            ListCodecErrorKind::TooShort,
        ));
    }
    if vec.len() > MAX {
        return DataResult::error(BoundedVec::<T, MIN, MAX>::error(
            vec,
            ListCodecErrorKind::TooLong,
        ));
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
                    return DataResult::error(Self::error(&elements, ListCodecErrorKind::TooShort));
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
