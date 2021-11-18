use std::fmt;

#[derive(Debug)]
pub enum Error {
    Csv(csv::Error),
    CsvIntoInnerError(csv::IntoInnerError<csv::Writer<Vec<u8>>>),
    Command(std::io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    IndexOutOfRange,
    VersionRangeEndError,
    VersionRangeStartError,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Csv(ref err) => write!(fmt, "{}", err),
            Error::CsvIntoInnerError(ref err) => write!(fmt, "{}", err),
            Error::Command(ref err) => write!(fmt, "{}", err),
            Error::FromUtf8Error(ref err) => write!(fmt, "{}", err),
            Error::IndexOutOfRange => write!(fmt, "The index out of range."),
            Error::VersionRangeEndError => write!(fmt, "The end of range have to be greater than the end of another range."),
            Error::VersionRangeStartError => write!(fmt, "The start of two ranges have to be the same."),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for Error {
    fn from(err: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Error {
        Error::CsvIntoInnerError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Command(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::FromUtf8Error(err)
    }
}
