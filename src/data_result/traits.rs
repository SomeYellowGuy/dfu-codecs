use crate::data_result::DataResult;

/// A type conversion from one type to this type that may fail, resulting in a [`DataResult`].
///
/// Always prefer using [`DataTryFrom`] over [`DataTryInto`] for implementing the conversion,
/// as an implementation of [`DataTryInto`] will automatically work as well.
pub trait DataTryFrom<T>: Sized {
    /// Performs the conversion.
    fn data_try_from(value: T) -> DataResult<Self>;
}

impl<T, U> DataTryInto<U> for T
where
    U: DataTryFrom<T>,
{
    #[inline]
    /// Calls `U::flat_try_from()`, which performs the conversion.
    fn flat_try_into(self) -> DataResult<U> {
        U::data_try_from(self)
    }
}

// Blanket implementation to always provide a successful result from T to U
// if U implements From<T>.
impl<T, U: From<T>> DataTryFrom<T> for U {
    fn data_try_from(value: T) -> DataResult<Self> {
        DataResult::success(U::from(value))
    }
}

/// A type conversion from this type to another that may fail, resulting in a [`DataResult`].
///
/// Always prefer using [`DataTryFrom`] over [`DataTryInto`] for implementing the conversion,
/// as an implementation of [`DataTryInto`] will automatically work as well.
pub trait DataTryInto<T>: Sized {
    /// Performs the conversion.
    fn flat_try_into(self) -> DataResult<T>;
}
