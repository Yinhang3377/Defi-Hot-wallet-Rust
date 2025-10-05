use core::fmt::{self, Debug, Display};
use serde::{de, ser};
use std::boxed::Box;

pub type Result<T> = core::result::Result<T, Error>;

pub struct Error {
    inner: Box<ErrorImpl>,
}

impl Error {
    fn new(inner: ErrorImpl) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    #[cfg(feature = "std")]
    pub(crate) fn io(error: std::io::Error) -> Self {
        Self::new(ErrorImpl::Io(error))
    }

    pub(crate) fn unexpected_eof() -> Self {
        Self::new(ErrorImpl::UnexpectedEof)
    }

    pub(crate) fn any_unsupported() -> Self {
        Self::new(ErrorImpl::AnyUnsupported)
    }

    pub(crate) fn invalid_utf8() -> Self {
        Self::new(ErrorImpl::InvalidUtf8)
    }

    pub(crate) fn invalid_char() -> Self {
        Self::new(ErrorImpl::InvalidChar)
    }

    pub(crate) fn sequence_length_required() -> Self {
        Self::new(ErrorImpl::SequenceLengthRequired)
    }

    pub(crate) fn map_length_required() -> Self {
        Self::new(ErrorImpl::MapLengthRequired)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self {
            inner: Box::new(ErrorImpl::Message(std::string::ToString::to_string(&msg))),
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self {
            inner: Box::new(ErrorImpl::Message(std::string::ToString::to_string(&msg))),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Category {
    Io,
    Data,
    Eof,
}

impl Category {
    /// Returns `true` if the category is [`Io`].
    ///
    /// [`Io`]: Category::Io
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io)
    }

    /// Returns `true` if the category is [`Data`].
    ///
    /// [`Data`]: Category::Data
    pub fn is_data(&self) -> bool {
        matches!(self, Self::Data)
    }

    /// Returns `true` if the category is [`Eof`].
    ///
    /// [`Eof`]: Category::Eof
    pub fn is_eof(&self) -> bool {
        matches!(self, Self::Eof)
    }
}

impl Error {
    pub fn classify(&self) -> Category {
        match self.inner.as_ref() {
            ErrorImpl::Message(_) => Category::Data,
            #[cfg(feature = "std")]
            ErrorImpl::Io(_) => Category::Io,
            ErrorImpl::UnexpectedEof => Category::Eof,
            ErrorImpl::AnyUnsupported
            | ErrorImpl::InvalidUtf8
            | ErrorImpl::InvalidChar
            | ErrorImpl::SequenceLengthRequired
            | ErrorImpl::MapLengthRequired => Category::Data,
        }
    }
}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        if let ErrorImpl::Io(error) = *error.inner {
            error
        } else {
            match error.classify() {
                Category::Io => unreachable!(),
                Category::Data => std::io::Error::new(std::io::ErrorKind::InvalidData, error),
                Category::Eof => std::io::Error::new(std::io::ErrorKind::UnexpectedEof, error),
            }
        }
    }
}

#[derive(Debug)]
enum ErrorImpl {
    Message(std::string::String),
    #[cfg(feature = "std")]
    Io(std::io::Error),
    UnexpectedEof,

    AnyUnsupported,

    InvalidUtf8,
    InvalidChar,

    SequenceLengthRequired,
    MapLengthRequired,
}

impl Display for ErrorImpl {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Message(msg) => formatter.write_str(msg),
            #[cfg(feature = "std")]
            ErrorImpl::Io(e) => Display::fmt(&e, formatter),
            ErrorImpl::UnexpectedEof => formatter.write_str("unexpected end of input"),
            ErrorImpl::AnyUnsupported => formatter.write_str("BARE does not support any"),
            ErrorImpl::InvalidUtf8 => formatter.write_str("invalid utf-8 in string"),
            ErrorImpl::InvalidChar => formatter.write_str("invalid unicode codepoint in char"),
            ErrorImpl::SequenceLengthRequired => formatter.write_str("sequence length required"),
            ErrorImpl::MapLengthRequired => formatter.write_str("map length required"),
        }
    }
}
