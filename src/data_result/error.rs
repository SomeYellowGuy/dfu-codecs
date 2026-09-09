use crate::codec::BuiltInError;
use smallvec::{SmallVec, smallvec};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};

type InnerDataError = SmallVec<[ErrorMessage; 3]>;

/// A list of error messages.
#[derive(Debug)]
pub struct DataError(pub(crate) InnerDataError);

impl DataError {
    pub fn new(message: ErrorMessage) -> Self {
        Self(smallvec![message])
    }

    pub fn push(&mut self, error: ErrorMessage) {
        self.0.push(error)
    }

    pub fn append(&mut self, mut other: DataError) {
        self.0.append(&mut other.0)
    }

    pub fn write_message(&self, f: &mut impl Write) -> std::fmt::Result {
        for (i, message) in self.0.iter().enumerate() {
            if i > 0 && !matches!(message, ErrorMessage::Additional(_)) {
                // Add a semicolon delimiter.
                write!(f, "; ")?;
            }
            write!(f, "{message}")?;
        }
        Ok(())
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.write_message(f)
    }
}

impl From<InnerDataError> for DataError {
    fn from(error: InnerDataError) -> Self {
        DataError(error)
    }
}

impl From<DataError> for InnerDataError {
    fn from(error: DataError) -> Self {
        error.0
    }
}

/// An error message. It is the smallest unit of an error stored by a [`DataResult`].
pub enum ErrorMessage {
    /// Stores a string value, which is preceded by a semicolon (`;`) for an entire `DataError`.
    String(Cow<'static, str>),
    /// Stores a value that implements the [`Error`] trait, which is preceded by a semicolon (`;`) for an entire `DataError`.
    Dynamic(Box<dyn Error>),
    /// Stores a built-in codec error.
    BuiltIn(BuiltInError),
    /// Stores a string value, which is not preceded by anything (directly continues from the last message) for an entire `DataError`.
    Additional(Cow<'static, str>),
}

impl Debug for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) | Self::Additional(s) => f.debug_tuple("String").field(s).finish(),
            Self::Dynamic(d) => f.debug_tuple("Dynamic").field(&d).finish(),
            Self::BuiltIn(e) => f.debug_tuple("BuiltIn").field(&e).finish(),
        }
    }
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) | Self::Additional(s) => write!(f, "{s}"),
            Self::Dynamic(d) => write!(f, "{d}"),
            Self::BuiltIn(e) => write!(f, "{e}"),
        }
    }
}

impl ErrorMessage {
    /// Creates a new *string* error message.
    pub fn new(string: impl Into<Cow<'static, str>>) -> Self {
        Self::String(string.into())
    }

    /// Creates a new *dynamic* error message.
    pub fn dynamic(display: impl Error + 'static) -> ErrorMessage {
        Self::Dynamic(Box::new(display))
    }

    /// Creates a new *additional* error message.
    pub fn additional(string: impl Into<Cow<'static, str>>) -> Self {
        Self::Additional(string.into())
    }
}

impl<T: Into<Cow<'static, str>>> From<T> for ErrorMessage {
    fn from(value: T) -> Self {
        ErrorMessage::new(value)
    }
}

impl From<BuiltInError> for ErrorMessage {
    fn from(value: BuiltInError) -> Self {
        ErrorMessage::BuiltIn(value)
    }
}
