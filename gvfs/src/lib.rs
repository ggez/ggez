//! TODO: Crate docs

#![forbid(missing_docs)]
#![forbid(missing_debug_implementations)]
#![forbid(unused_results)]
#![forbid(unsafe_code)]
#![warn(bare_trait_objects)]
#![warn(missing_copy_implementations)]

use std::fmt;
use std::sync::Arc;

pub mod filesystem;
pub mod vfs;

/// Error type
///
/// TODO: This all needs to be consistent-ified.
///
/// error types: invalid vfs type (zip file corrupt, etc),
/// read/write error (IOError), not found, maybe something else...
#[derive(Debug, Clone)]
pub enum Error {
    /// TODO
    VfsError(String),
    /// TODO
    ResourceNotFound(String, Vec<(std::path::PathBuf, Error)>),
    /// TODO
    ZipError(String),
    /// TODO
    IOError(Arc<std::io::Error>),
}

/// Shortcut result type
pub type Result<T = ()> = std::result::Result<T, Error>;

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Error {
        let errstr = {
            use std::error::Error;
            format!("Zip error: {}", e.description())
        };
        Error::VfsError(errstr)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOError(Arc::new(e))
    }
}

// TODO
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _ => write!(f, "Error {:?}", self),
        }
    }
}

// TODO
impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
