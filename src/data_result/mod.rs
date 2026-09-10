mod assertion;
mod error;
mod traits;

use crate::lifecycle::Lifecycle;
pub use error::{DataError, ErrorMessage};
pub use smallvec::smallvec;
use std::fmt::Debug;
use smallvec::SmallVec;
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
    Error {
        partial: Option<R>,
        error: DataError,
    },
}

impl<R> DataResult<R> {
    fn new(kind: DataResultKind<R>, lifecycle: Lifecycle) -> Self {
        Self { kind, lifecycle }
    }

    /// Creates a new *success* data result with the default lifecycle.
    pub fn success(success: R) -> Self {
        Self::success_with_lifecycle(success, Lifecycle::Experimental)
    }

    /// Creates a new *success* data result with the provided lifecycle.
    pub fn success_with_lifecycle(success: R, lifecycle: Lifecycle) -> Self {
        Self::new(DataResultKind::Success(success), lifecycle)
    }

    /// Creates a new *failed* data result with the provided error and the default lifecycle.
    pub fn error(message: impl Into<ErrorMessage>) -> Self {
        Self::error_with_lifecycle(message, Lifecycle::Experimental)
    }

    /// Creates a new *failed* data result with the provided error and lifecycle.
    pub fn error_with_lifecycle(message: impl Into<ErrorMessage>, lifecycle: Lifecycle) -> Self {
        Self::new(
            DataResultKind::Error {
                partial: None,
                error: DataError::new(message.into()),
            },
            lifecycle,
        )
    }

    /// Creates a new *partial* data result with the provided error and the default lifecycle.
    pub fn partial(value: R, message: impl Into<ErrorMessage>) -> Self {
        Self::partial_with_lifecycle(value, message, Lifecycle::Experimental)
    }

    /// Creates a new *partial* data result with the provided error and lifecycle.
    pub fn partial_with_lifecycle(
        value: R,
        message: impl Into<ErrorMessage>,
        lifecycle: Lifecycle,
    ) -> Self {
        Self::new(
            DataResultKind::Error {
                partial: Some(value),
                error: DataError::new(message.into()),
            },
            lifecycle,
        )
    }

    /// Adds the given lifecycle to that of this result.
    pub fn add_lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle += lifecycle;
        self
    }

    /// Replaces the lifecycle of this result with lifecycle.
    pub fn with_lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Replaces the partial value of the result (if it is an error one) with `value`.
    pub fn with_partial(mut self, value: R) -> Self {
        if let DataResultKind::Error { partial, .. } = &mut self.kind {
            *partial = Some(value);
        }
        self
    }

    /// Returns whether this result is a success result.
    pub fn is_success(&self) -> bool {
        matches!(self.kind, DataResultKind::Success(_))
    }

    /// Returns whether this result is a success or partial result.
    pub fn is_success_or_partial(&self) -> bool {
        matches!(
            self.kind,
            DataResultKind::Success(_)
                | DataResultKind::Error {
                    partial: Some(_),
                    ..
                }
        )
    }

    /// Returns whether this result is an error (partial or failed) result.
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Consumes this result, returning its value wrapped in a `Some` if it is a success. Otherwise,
    /// this returns `None`.
    pub fn into_success(self) -> Option<R> {
        match self.kind {
            DataResultKind::Success(success) => Some(success),
            _ => None,
        }
    }

    /// Consumes this result, returning its value wrapped in a `Some` if it is a success or partial. Otherwise,
    /// this returns `None`.
    pub fn into_success_or_partial(self) -> Option<R> {
        match self.kind {
            DataResultKind::Success(success) => Some(success),
            DataResultKind::Error { partial, .. } => partial,
        }
    }

    /// Consumes this result, returning the reference to its value wrapped in a `Some` if it is a success.
    /// Otherwise, this returns `None`.
    pub fn success_ref(&self) -> Option<&R> {
        match &self.kind {
            DataResultKind::Success(success) => Some(success),
            _ => None,
        }
    }

    /// Otherwise, this returns `None`.
    /// Consumes this result, returning the reference to its value wrapped in a `Some` if it is a success or partial.
    pub fn success_or_partial_ref(&self) -> Option<&R> {
        match &self.kind {
            DataResultKind::Success(success) => Some(success),
            DataResultKind::Error { partial, .. } => partial.as_ref(),
        }
    }

    /// Extracts the value (if any) of the result, leaving behind a data result of the unit tuple `()`
    /// that keeps all other info (errors and lifecycle) that was present on this result.
    pub fn extract(self) -> (Option<R>, DataResult<()>) {
        match self.kind {
            DataResultKind::Success(r) => (
                Some(r),
                DataResult::new(DataResultKind::Success(()), self.lifecycle),
            ),
            DataResultKind::Error { partial, error } => {
                let new_partial = partial.as_ref().map(|_| ());
                (
                    partial,
                    DataResult::new(
                        DataResultKind::Error {
                            partial: new_partial,
                            error,
                        },
                        self.lifecycle,
                    ),
                )
            }
        }
    }

    /// Converts this result to a [`DataResultStatus`].
    pub fn status(self) -> DataResultStatus<R> {
        match self {
            Self {
                kind: DataResultKind::Success(r),
                ..
            } => DataResultStatus::Success(r),
            Self {
                kind:
                    DataResultKind::Error {
                        partial: Some(r), ..
                    },
                ..
            } => DataResultStatus::Partial(r),
            Self {
                kind: DataResultKind::Error { partial: None, .. },
                ..
            } => DataResultStatus::Failed,
        }
    }

    /// Returns the contained value if this result is a success, consuming it.
    ///
    /// # Panics
    ///
    /// Panics if the self value is an error (partial or failed result).
    pub fn unwrap(self) -> R {
        self.into_success().expect("DataResult should be a success")
    }

    /// Returns the contained value if this result is a success or partial, consuming it.
    ///
    /// # Panics
    ///
    /// Panics if the self value is a failed result.
    pub fn unwrap_or_partial(self) -> R {
        self.into_success_or_partial()
            .expect("DataResult should be a success or partial")
    }

    /// Separates the value of this result (if any), returning it, along with adding
    /// its errors to `vec`.
    pub fn separate(self, vec: &mut DataError) -> Option<R> {
        let (value, error) = match self.kind {
            DataResultKind::Success(success) => (Some(success), None),
            DataResultKind::Error { partial, error } => (partial, Some(error)),
        };
        if let Some(error) = error {
            vec.append(error)
        }
        value
    }

    /// Maps the value inside a success or partial result using `f`.
    pub fn map<T>(self, f: impl FnOnce(R) -> T) -> DataResult<T> {
        let kind = match self.kind {
            DataResultKind::Success(value) => DataResultKind::Success(f(value)),

            DataResultKind::Error {
                error,
                partial: Some(value),
            } => DataResultKind::Error {
                error,
                partial: Some(f(value)),
            },

            DataResultKind::Error { error, .. } => DataResultKind::Error {
                error,
                partial: None,
            },
        };
        DataResult::new(kind, self.lifecycle)
    }

    /// Maps the value inside a success using `success_function` if this result is a success, or
    /// maps its error data and the partial value using `error_function`.
    pub fn map_or_else<T>(
        self,
        error_function: impl FnOnce(Option<R>, DataError) -> T,
        success_function: impl FnOnce(R) -> T,
    ) -> T {
        match self.kind {
            DataResultKind::Success(value) => success_function(value),
            DataResultKind::Error { partial, error } => error_function(partial, error),
        }
    }

    /// Calls `f` for the new message to add to this result if it is an error.
    pub fn add_message_if_error(mut self, f: impl FnOnce() -> ErrorMessage) -> Self {
        if let DataResultKind::Error { error, .. } = &mut self.kind {
            error.push(f());
        }
        self
    }

    /// Maps the value inside this result if it is a success or partial using `f`,
    /// combining the value partiality (if any), errors and lifecycles of this and the result from `f`.
    pub fn and_then<T>(self, f: impl FnOnce(R) -> DataResult<T>) -> DataResult<T> {
        match self.kind {
            DataResultKind::Success(value) => f(value).add_lifecycle(self.lifecycle),

            DataResultKind::Error {
                mut error,
                partial: Some(value),
            } => {
                let function_result = f(value);
                let kind = match function_result.kind {
                    DataResultKind::Success(value) => DataResultKind::Error {
                        error,
                        partial: Some(value),
                    },

                    DataResultKind::Error {
                        error: next_error,
                        partial,
                    } => {
                        error.append(next_error);
                        DataResultKind::Error { partial, error }
                    }
                };
                DataResult::new(kind, self.lifecycle + function_result.lifecycle)
            }

            DataResultKind::Error { error, .. } => DataResult::new(
                DataResultKind::Error {
                    partial: None,
                    error,
                },
                self.lifecycle,
            ),
        }
    }

    /// Combines the values inside 2 results, applying them in `f` if all results have values.
    ///
    /// - If both results are successful, the returned one is also a success.
    /// - If at least one of the results is not a success, and neither result is failed, the returned one is a partial.
    /// - Otherwise, the returned one is a failed result.
    pub fn apply_2<A, B>(f: impl FnOnce(A, B) -> R,
                         a: DataResult<A>, b: DataResult<B>,
    ) -> DataResult<R> {
        let resultant_lifecycle = a.lifecycle + b.lifecycle;
        if a.is_success() && b.is_success() {
            return DataResult::success_with_lifecycle(f(a.unwrap(), b.unwrap()), resultant_lifecycle);
        }

        let mut error = DataError(SmallVec::new());
        let a = a.separate(&mut error);
        let b = b.separate(&mut error);

        let partial = match (a, b) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        };

        Self::new(DataResultKind::Error { partial, error }, resultant_lifecycle)
    }

    /// Combines the values inside 2 results, applying them in `f` if all results have values.
    /// - If all results are successful, the returned one is also a success.
    /// - If not all results are successful, and no results are failed, the returned one is a partial.
    /// - Otherwise, the returned one is a failed result.
    ///
    /// In addition to the above, the returned result is also marked as [`Lifecycle::Stable`].
    pub fn apply_2_stable<A, B>(
        f: impl FnOnce(A, B) -> R,
        a: DataResult<A>,
        b: DataResult<B>,
    ) -> DataResult<R> {
        Self::apply_2(f, a, b).with_lifecycle(Lifecycle::Stable)
    }

    /// Replaces the data of this result with `value` (if this result is a success or partial).
    ///
    /// This is equivalent to `result.map(_ -> value).setPartial(value)` in Java.
    pub fn with_data<T>(self, value: T) -> DataResult<T> {
        match self {
            DataResult {
                kind: DataResultKind::Success(_),
                lifecycle,
            } => DataResult::success_with_lifecycle(value, lifecycle),
            DataResult {
                kind: DataResultKind::Error { error, .. },
                lifecycle,
            } => DataResult {
                kind: DataResultKind::Error {
                    partial: Some(value),
                    error,
                },
                lifecycle,
            },
        }
    }

    /// Creates and returns the message of this result, which can be displayed anywhere needed.
    pub fn message(&self) -> Option<String> {
        if let DataResultKind::Error { error, .. } = &self.kind {
            Some(error.to_string())
        } else {
            None
        }
    }

    /// Consumes this result, returning its [`DataError`].
    pub fn data_error(self) -> Option<DataError> {
        match self.kind {
            DataResultKind::Error { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! apply_data_results {
    ($vis:vis $name:ident | $count:literal | $($generic:ident $results:ident),+) => {
        #[doc = concat!(
            "Combines the values inside ", $count, " results, applying them in `f` if all results have values.\n\n",
            " - If all results are successful, the returned one is also a success.\n",
            " - If not all results are a success, and no results are failed, the returned one is a partial.\n",
            " - Otherwise, the returned one is a failed result."
        )]
        #[allow(clippy::too_many_arguments)]
        $vis fn $name< $($generic),+ >(f: impl FnOnce( $($generic),+ ) -> R,
            $( $results: DataResult<$generic> ),+
        ) -> DataResult<R> {
            let resultant_lifecycle = $crate::Lifecycle::add_all([ $( $results.lifecycle ),+ ]).unwrap();
            if $( $results.is_success() )&&+ {
                return DataResult::success_with_lifecycle(f(
                    $( $results.unwrap() ),+
                ), resultant_lifecycle);
            }

            let mut error = DataError(smallvec![]);
            $( let $results = $results.separate(&mut error); )+

            let partial = match ( $($results,)+ ) {
                ( $(Some($results)),+ ) => Some(f( $( $results ),+ )),
                _ => None,
            };

            Self::new(DataResultKind::Error {partial, error}, resultant_lifecycle)
        }
    };
}

// Include the generated apply_n files.
include!(concat!(env!("OUT_DIR"), "/generated/data_result.rs"));
