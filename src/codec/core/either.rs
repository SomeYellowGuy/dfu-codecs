use crate::codec::{BuiltInError, Decode, Encode};
use crate::{DataResult, DynamicOps};
use either::Either;

impl<L: Encode, R: Encode> Encode for Either<L, R> {
    fn encode<O: DynamicOps>(&self, ops: &O, prefix: Option<O::Value>) -> DataResult<O::Value> {
        match self {
            Self::Left(l) => l.encode(ops, prefix),
            Self::Right(r) => r.encode(ops, prefix),
        }
    }
}

impl<L: Decode, R: Decode> Decode for Either<L, R> {
    fn decode<O: DynamicOps>(ops: &O, input: &O::Value) -> DataResult<Self> {
        let first_read = L::decode(ops, input).map(Either::Left);
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
            #[expect(
                clippy::unwrap_used,
                reason = "the result is guaranteed to have a data error"
            )]
            Box::new(first_read.data_error().unwrap()),
            #[expect(
                clippy::unwrap_used,
                reason = "the result is guaranteed to have a data error"
            )]
            Box::new(second_read.data_error().unwrap()),
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::json_ops::JsonOps;
    use crate::{assert_decode_error, assert_decode_success, assert_encode_success};
    use either::Either;
    use serde_json::json;

    #[test]
    fn simple() {
        assert_encode_success!(JsonOps, Either::<i32, String>::Left(5) => json!(5));
        assert_encode_success!(
            JsonOps,
            Either::<i32, String>::Right("I am some text.".to_string())
                => json!("I am some text.")
        );

        assert_decode_success!(JsonOps, Either<i32, String>, json!(-238) => Either::Left(-238));
        assert_decode_error!(JsonOps, Either<u32, String>, json!(-238) => "Failed to parse either. First: Could not fit i32 to u32: -238; Second: Not a string: -238");
        assert_decode_success!(JsonOps, Either<u32, String>, json!("hello") => Either::Right("hello".to_string()));
        assert_decode_error!(JsonOps, Either<u32, String>, json!(true) => "Failed to parse either. First: Not a number: true; Second: Not a string: true");
    }
}
