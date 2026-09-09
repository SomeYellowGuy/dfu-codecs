use crate::codec::{BuiltInError, Decode, Encode};
use crate::{DataResult, DynamicOps};
use either::Either;

impl<L: Encode, R: Encode> Encode for Either<L, R> {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: O::Value) -> DataResult<O::Value> {
        match self {
            Self::Left(l) => l.encode(ops, prefix),
            Self::Right(r) => r.encode(ops, prefix),
        }
    }
}

impl<L: Decode, R: Decode> Decode for Either<L, R> {
    fn decode<O: DynamicOps>(ops: &O, input: O::Value) -> DataResult<Self> {
        let first_read = L::decode(ops, input.clone()).map(Either::Left);
        if first_read.is_success() {
            return first_read;
        }
        let second_read = R::decode(ops, input).map(Either::Right);
        if second_read.is_success() {
            return second_read;
        }
        if first_read.is_success_or_partial() {
            return first_read;
        }
        if second_read.is_success_or_partial() {
            return second_read;
        }
        DataResult::error(BuiltInError::FailedToParseEither(
            Box::new(first_read.data_error().unwrap()),
            Box::new(second_read.data_error().unwrap()),
        ))
    }
}
