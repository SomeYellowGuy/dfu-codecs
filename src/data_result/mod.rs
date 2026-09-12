mod assertion;
mod error;
mod traits;

use crate::lifecycle::Lifecycle;
pub use error::{DataError, ErrorMessage};
use std::fmt::Debug;
pub use traits::{DataTryFrom, DataTryInto};

/// A result of encoding or decoding something.
///
/// This result can be classified into three types:
/// - **Success** results: they have a value of their type.
/// - **Error** results: they store error data (to indicate what was wrong during a process). They can be further classified into:
///   - *Partial* results: they have a value of their type *and* the error data.
///   - *Failed* results: they only have the error data.
///
/// In addition, all `DataResult`s store a [`Lifecycle`] marker for their data.
#[derive(Debug)]
#[must_use = "this `DataResult` may be a failed or partial one, which should be handled"]
pub struct DataResult<R> {
    pub kind: DataResultKind<R>,
    pub lifecycle: Lifecycle,
}

/// A subset of a [`DataResult`] that only stores the value of its type.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DataResultStatus<R> {
    Success(R),
    Partial(R),
    Failed,
}

/// A type of [`DataResult`], storing result-specific data.
#[derive(Debug)]
pub enum DataResultKind<R> {
    Success(R),
    Partial { value: R, error: DataError },
    Failed(DataError),
}

impl<R> DataResult<R> {
    #[inline]
    const fn new(kind: DataResultKind<R>, lifecycle: Lifecycle) -> Self {
        Self { kind, lifecycle }
    }

    /// Creates a new *successful* data result with the default lifecycle.
    #[inline]
    pub const fn success(success: R) -> Self {
        Self::success_with_lifecycle(success, Lifecycle::Experimental)
    }

    /// Creates a new *successful* data result with the provided lifecycle.
    #[inline]
    pub const fn success_with_lifecycle(success: R, lifecycle: Lifecycle) -> Self {
        Self::new(DataResultKind::Success(success), lifecycle)
    }

    /// Creates a new *failed* data result with the provided error and the default lifecycle.
    #[inline]
    pub fn error(message: impl Into<ErrorMessage>) -> Self {
        Self::error_with_lifecycle(message, Lifecycle::Experimental)
    }

    /// Creates a new *failed* data result with the provided error and lifecycle.
    #[inline]
    pub fn error_with_lifecycle(message: impl Into<ErrorMessage>, lifecycle: Lifecycle) -> Self {
        Self::new(
            DataResultKind::Failed(DataError::new(message.into())),
            lifecycle,
        )
    }

    /// Creates a new *partial* data result with the provided error and the default lifecycle.
    #[inline]
    pub fn partial(message: impl Into<ErrorMessage>, value: R) -> Self {
        Self::partial_with_lifecycle(message, value, Lifecycle::Experimental)
    }

    /// Creates a new *partial* data result with the provided error and lifecycle.
    #[inline]
    pub fn partial_with_lifecycle(
        message: impl Into<ErrorMessage>,
        value: R,
        lifecycle: Lifecycle,
    ) -> Self {
        Self::new(
            DataResultKind::Partial {
                value,
                error: DataError::new(message.into()),
            },
            lifecycle,
        )
    }

    /// Adds the given lifecycle to that of this result.
    #[inline]
    pub fn add_lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle += lifecycle;
        self
    }

    /// Replaces the lifecycle of this result with lifecycle.
    #[inline]
    pub const fn with_lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Replaces the partial value of the result (if it is an error one) with `value`.
    #[inline]
    pub fn with_partial(mut self, value: R) -> Self {
        if let DataResultKind::Partial { error, .. } | DataResultKind::Failed(error) = self.kind {
            self.kind = DataResultKind::Partial { value, error }
        }
        self
    }

    /// Returns whether this result is a success result.
    #[inline]
    pub const fn is_success(&self) -> bool {
        matches!(self.kind, DataResultKind::Success(_))
    }

    /// Returns whether this result is a success or partial result.
    #[inline]
    pub const fn is_success_or_partial(&self) -> bool {
        matches!(
            self.kind,
            DataResultKind::Success(_) | DataResultKind::Partial { .. }
        )
    }

    /// Returns whether this result is an error (partial or failed) result.
    #[inline]
    pub const fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Consumes this result, returning its value wrapped in a `Some` if it is a success. Otherwise,
    /// this returns `None`.
    #[inline]
    pub fn into_success(self) -> Option<R> {
        match self.kind {
            DataResultKind::Success(success) => Some(success),
            _ => None,
        }
    }

    /// Consumes this result, returning its value wrapped in a `Some` if it is a success or partial. Otherwise,
    /// this returns `None`.
    #[inline]
    pub fn into_success_or_partial(self) -> Option<R> {
        match self.kind {
            DataResultKind::Success(success) => Some(success),
            DataResultKind::Partial { value, .. } => Some(value),
            DataResultKind::Failed(_) => None,
        }
    }

    /// Consumes this result, returning the reference to its value wrapped in a `Some` if it is a success.
    /// Otherwise, this returns `None`.
    #[inline]
    pub const fn success_ref(&self) -> Option<&R> {
        match &self.kind {
            DataResultKind::Success(success) => Some(success),
            _ => None,
        }
    }

    /// Otherwise, this returns `None`.
    /// Consumes this result, returning the reference to its value wrapped in a `Some` if it is a success or partial.
    #[inline]
    pub const fn success_or_partial_ref(&self) -> Option<&R> {
        match &self.kind {
            DataResultKind::Success(success) => Some(success),
            DataResultKind::Partial { value, .. } => Some(value),
            DataResultKind::Failed(_) => None,
        }
    }

    /// Extracts the value (if any) of the result, leaving behind a data result of the unit tuple `()`
    /// that keeps all other info (errors and lifecycle) that was present on this result.
    #[inline]
    pub fn extract(self) -> (Option<R>, DataResult<()>) {
        match self.kind {
            DataResultKind::Success(r) => (
                Some(r),
                DataResult::new(DataResultKind::Success(()), self.lifecycle),
            ),
            DataResultKind::Partial { value, error } => (
                Some(value),
                DataResult::new(DataResultKind::Partial { value: (), error }, self.lifecycle),
            ),
            DataResultKind::Failed(error) => (
                None,
                DataResult::new(DataResultKind::Failed(error), self.lifecycle),
            ),
        }
    }

    /// Converts this result to a [`DataResultStatus`].
    #[inline]
    pub fn status(self) -> DataResultStatus<R> {
        match self.kind {
            DataResultKind::Success(value) => DataResultStatus::Success(value),
            DataResultKind::Partial { value, .. } => DataResultStatus::Partial(value),
            DataResultKind::Failed(_) => DataResultStatus::Failed,
        }
    }

    /// Returns the contained value if this result is a success, consuming it.
    ///
    /// # Panics
    ///
    /// Panics if the self value is an error (partial or failed result).
    #[inline]
    pub fn unwrap(self) -> R {
        self.into_success().expect("DataResult should be a success")
    }

    /// Returns the contained value if this result is a success or partial, consuming it.
    ///
    /// # Panics
    ///
    /// Panics if the self value is a failed result.
    #[inline]
    pub fn unwrap_or_partial(self) -> R {
        self.into_success_or_partial()
            .expect("DataResult should be a success or partial")
    }

    /// Separates the value of this result (if any), returning it, along with adding
    /// its errors to `vec`.
    #[inline]
    pub fn separate(self, vec: &mut DataError) -> Option<R> {
        let (value, error) = match self.kind {
            DataResultKind::Success(value) => (Some(value), None),
            DataResultKind::Partial { value, error } => (Some(value), Some(error)),
            DataResultKind::Failed(error) => (None, Some(error)),
        };
        if let Some(error) = error {
            vec.append(error);
        }
        value
    }

    /// Maps the value inside a success or partial result using `f`.
    #[inline]
    pub fn map<T>(self, f: impl FnOnce(R) -> T) -> DataResult<T> {
        let kind = match self.kind {
            DataResultKind::Success(value) => DataResultKind::Success(f(value)),

            DataResultKind::Partial { value, error } => DataResultKind::Partial {
                error,
                value: f(value),
            },

            DataResultKind::Failed(error) => DataResultKind::Failed(error),
        };
        DataResult::new(kind, self.lifecycle)
    }

    /// Maps the value inside a success using `success_function` if this result is a success, or
    /// maps its error data and the partial value using `error_function`.
    #[inline]
    pub fn map_or_else<T>(
        self,
        error_function: impl FnOnce(Option<R>, DataError) -> T,
        success_function: impl FnOnce(R) -> T,
    ) -> T {
        match self.kind {
            DataResultKind::Success(value) => success_function(value),
            DataResultKind::Partial { value, error } => error_function(Some(value), error),
            DataResultKind::Failed(error) => error_function(None, error),
        }
    }

    /// Calls `f` for the new message to add to this result if it is an error.
    #[inline]
    pub fn add_message_if_error(mut self, f: impl FnOnce() -> ErrorMessage) -> Self {
        if let DataResultKind::Failed(error) | DataResultKind::Partial { error, .. } =
            &mut self.kind
        {
            error.push(f());
        }
        self
    }

    /// Maps the value inside this result if it is a success or partial using `f`,
    /// combining the value partiality (if any), errors and lifecycles of this and the result from `f`.
    #[inline]
    pub fn and_then<T>(self, f: impl FnOnce(R) -> DataResult<T>) -> DataResult<T> {
        match self.kind {
            DataResultKind::Success(value) => f(value).add_lifecycle(self.lifecycle),

            DataResultKind::Partial { mut error, value } => {
                let function_result = f(value);
                let kind = match function_result.kind {
                    DataResultKind::Success(value) => DataResultKind::Partial { error, value },

                    DataResultKind::Partial {
                        error: next_error,
                        value,
                    } => {
                        error.append(next_error);
                        DataResultKind::Partial { value, error }
                    }

                    DataResultKind::Failed(next_error) => {
                        error.append(next_error);
                        DataResultKind::Failed(error)
                    }
                };
                DataResult::new(kind, self.lifecycle + function_result.lifecycle)
            }

            DataResultKind::Failed(error) => {
                DataResult::new(DataResultKind::Failed(error), self.lifecycle)
            }
        }
    }

    /// Combines the values inside 2 results, applying them in `f` if all results have values.
    ///
    /// - If both results are successes, the returned one is also a success.
    /// - If at least one of the results is not a success, and neither result is failed, the returned one is a partial.
    /// - Otherwise, the returned one is a failed result.
    #[inline]
    pub fn apply_2<A, B>(
        f: impl FnOnce(A, B) -> R,
        a: DataResult<A>,
        b: DataResult<B>,
    ) -> DataResult<R> {
        let resultant_lifecycle = a.lifecycle + b.lifecycle;
        if a.is_success() && b.is_success() {
            return DataResult::success_with_lifecycle(
                f(a.unwrap(), b.unwrap()),
                resultant_lifecycle,
            );
        }

        let mut error = DataError::empty();
        let b = b.separate(&mut error);
        let a = a.separate(&mut error);

        let kind = match (a, b) {
            (Some(a), Some(b)) => DataResultKind::Partial {
                value: f(a, b),
                error,
            },
            _ => DataResultKind::Failed(error),
        };

        Self::new(kind, resultant_lifecycle)
    }

    /// Combines the values inside 2 results, applying them in `f` if all results have values.
    /// - If all results are successes, the returned one is also a success.
    /// - If not all results are successes, and no results are failed, the returned one is a partial.
    /// - Otherwise, the returned one is a failed result.
    ///
    /// In addition to the above, the returned result is also marked as [`Lifecycle::Stable`].
    #[inline]
    pub fn apply_2_stable<A, B>(
        f: impl FnOnce(A, B) -> R,
        a: DataResult<A>,
        b: DataResult<B>,
    ) -> DataResult<R> {
        Self::apply_2(f, a, b).with_lifecycle(Lifecycle::Stable)
    }

    /// Replaces the data of this result with `value`.
    ///
    /// This is equivalent to `result.map(_ -> value).setPartial(value)` in Java.
    #[inline]
    pub fn with_data<T>(self, value: T) -> DataResult<T> {
        match self {
            DataResult {
                kind: DataResultKind::Success(_),
                lifecycle,
            } => DataResult::success_with_lifecycle(value, lifecycle),
            DataResult {
                kind: DataResultKind::Partial { error, .. } | DataResultKind::Failed(error),
                lifecycle,
            } => DataResult {
                kind: DataResultKind::Partial { value, error },
                lifecycle,
            },
        }
    }

    /// Creates and returns the message of this result, which can be displayed anywhere needed.
    #[inline]
    pub fn message(&self) -> Option<String> {
        if let DataResultKind::Partial { error, .. } | DataResultKind::Failed(error) = &self.kind {
            Some(error.to_string())
        } else {
            None
        }
    }

    /// Consumes this result, returning its [`DataError`].
    #[inline]
    pub fn data_error(self) -> Option<DataError> {
        match self.kind {
            DataResultKind::Partial { error, .. } | DataResultKind::Failed(error) => Some(error),
            DataResultKind::Success(_) => None,
        }
    }
}

#[macro_export]
macro_rules! apply_data_results {
    ($vis:vis $name:ident | $count:literal | $($generic:ident $results:ident),+ | $($separated_results:ident),+  $(| $too_many_arguments:ident)?) => {
        #[doc = concat!(
            "Combines the values inside ", $count, " results, applying them in `f` if all results have values.\n\n",
            " - If all results are successes, the returned one is also a success.\n",
            " - If not all results are a success, and no results are failed, the returned one is a partial.\n",
            " - Otherwise, the returned one is a failed result."
        )]
        $(#[expect(clippy::$too_many_arguments, reason = "the nature of the function requires taking many generic types")])?
        $vis fn $name< $($generic),+ >(f: impl FnOnce( $($generic),+ ) -> R,
            $( $results: DataResult<$generic> ),+
        ) -> DataResult<R> {
            let resultant_lifecycle = $crate::Lifecycle::add_all([ $( $results.lifecycle ),+ ]).unwrap();
            if $( $results.is_success() )&&+ {
                return DataResult::success_with_lifecycle(f(
                    $( $results.unwrap() ),+
                ), resultant_lifecycle);
            }

            let mut error = DataError::empty();
            // Error messages are actually applied in reverse (last to first result)!
            $( let $separated_results = $separated_results.separate(&mut error); )+

            let kind = match ( $($results,)+ ) {
                ( $(Some($results)),+ ) => DataResultKind::Partial { value: f( $( $results ),+ ), error },
                _ => DataResultKind::Failed(error),
            };

            Self::new(
                kind,
                resultant_lifecycle,
            )
        }
    };
}

// Include the generated apply_n files.
include!(concat!(env!("OUT_DIR"), "/generated/data_result.rs"));
