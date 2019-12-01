use std::fmt;

#[derive(Debug)]
pub enum Error {
    Regex(regex::Error),
    Csv(csv::Error),
    NoneCapturesError,
    TerminalWidthError,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Regex(ref err) => write!(fmt, "{}", err),
            Error::Csv(ref err) => write!(fmt, "{}", err),
            Error::NoneCapturesError => write!(fmt, "Can not capture groups."),
            Error::TerminalWidthError => write!(fmt, "Can not get the terminal size."),
        }
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error::Regex(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}
