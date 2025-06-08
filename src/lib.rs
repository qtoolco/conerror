#![doc = include_str!("../README.md")]

use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::ptr;

pub use conerror_macro::conerror;

pub type Result<T> = std::result::Result<T, Error>;

use inner::*;

#[cfg(feature = "send_sync")]
mod inner {
    pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

    pub trait ErrorTrait: std::error::Error + Send + Sync {}

    impl<T: std::error::Error + Send + Sync> ErrorTrait for T {}
}

#[cfg(not(feature = "send_sync"))]
mod inner {
    pub type BoxError = Box<dyn std::error::Error>;

    pub trait ErrorTrait: std::error::Error {}

    impl<T: std::error::Error> ErrorTrait for T {}
}

/// Represents an error with additional location information.
pub struct Error(Box<Inner>);

impl Error {
    /// Creates a new [Error] with location information.
    ///
    /// # Parameters
    ///
    /// - `error`: The error to wrap.
    /// - `file`: The file where the error occurred.
    /// - `line`: The line number where the error occurred.
    /// - `func`: The function where the error occurred.
    /// - `module`: The module where the error occurred.
    pub fn new<T>(
        error: T,
        file: &'static str,
        line: u32,
        func: &'static str,
        module: &'static str,
    ) -> Self
    where
        T: Into<BoxError>,
    {
        Self(Box::new(Inner {
            source: error.into(),
            location: Some(vec![Location {
                file,
                line,
                func,
                module,
            }]),
            context: Vec::new(),
        }))
    }

    /// Creates a new [Error] without location information.
    pub fn plain<T>(error: T) -> Self
    where
        T: Into<BoxError>,
    {
        Self(Box::new(Inner {
            source: error.into(),
            location: None,
            context: Vec::new(),
        }))
    }

    /// Chains an error with additional location information.
    ///
    /// If the provided error is not of type [Error], it creates a new [Error] with location information.
    /// If the provided error is of type [Error], it adds the location information if it was not created by [Error::plain].
    ///
    /// # Parameters
    ///
    /// - `error`: The error to wrap.
    /// - `file`: The file where the error occurred.
    /// - `line`: The line number where the error occurred.
    /// - `func`: The function where the error occurred.
    /// - `module`: The module where the error occurred.
    pub fn chain<T>(
        error: T,
        file: &'static str,
        line: u32,
        func: &'static str,
        module: &'static str,
    ) -> Self
    where
        T: ErrorTrait + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<Self>() {
            let error = ManuallyDrop::new(error);
            // SAFETY: type checked
            let mut error = unsafe { ptr::read(&error as *const _ as *const Self) };
            if let Some(ref mut location) = error.0.location {
                location.push(Location {
                    file,
                    line,
                    func,
                    module,
                });
            }
            return error;
        }

        Self::new(error, file, line, func, module)
    }

    pub fn context(mut self, context: impl ToString) -> Self {
        self.0.context.push(context.to_string());
        self
    }

    /// Returns the location information.
    pub fn location(&self) -> Option<&[Location]> {
        self.0.location.as_ref().map(|v| v.as_slice())
    }

    /// Returns the error message
    pub fn message(&self) -> String {
        let mut s = String::with_capacity(self.0.context.len() * 16);
        for c in self.0.context.iter().rev() {
            s.push_str(c);
            s.push_str(": ");
        }
        s.push_str(&self.0.source.to_string());
        s
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("Error", 2)?;
        s.serialize_field("message", &self.message())?;
        let location = self
            .0
            .location
            .as_ref()
            .map(|v| v.iter().map(Location::to_string).collect())
            .unwrap_or(Vec::new());
        s.serialize_field("location", &location)?;
        s.end()
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("source", &self.0.source)
            .field("location", &self.0.location)
            .field("context", &self.0.context)
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in self.0.context.iter().rev() {
            write!(f, "{}: ", c)?;
        }
        Display::fmt(&self.0.source, f)?;
        if let Some(ref location) = self.0.location {
            for (i, v) in location.iter().enumerate() {
                write!(f, "\n#{} {}", i, v)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.0.source)
    }
}

struct Inner {
    source: BoxError,
    location: Option<Vec<Location>>,
    context: Vec<String>,
}

/// Represents the location where an error occurred.
#[derive(Debug)]
pub struct Location {
    pub file: &'static str,
    pub line: u32,
    pub func: &'static str,
    /// Module path for function, struct/trait name for method.
    pub module: &'static str,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} {}::{}()",
            self.file, self.line, self.module, self.func
        )
    }
}
