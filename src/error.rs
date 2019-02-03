macro_rules! verify {
    ($condition:expr, $err:expr) => {
        if !$condition {
            return Err($err);
        }
    };
}

#[derive(Debug)]
pub enum Error {
    Invalid,
    InvalidLength,
    InvalidHeader,
    InvalidEntry,
    OutOfBounds,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                Error::Invalid => "Invalid WAD file",
                Error::InvalidLength => "Invalid WAD file length",
                Error::InvalidHeader => "Invalid WAD file header",
                Error::InvalidEntry => "Invalid WAD file entry",
                Error::OutOfBounds => "Index out of bounds",
            }
        )
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum LoadError {
    Error(Error),
    IoError(std::io::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoadError::Error(e) => write!(fmt, "{}", e),
            LoadError::IoError(e) => write!(fmt, "{}", e),
        }
    }
}

impl std::error::Error for LoadError {}
