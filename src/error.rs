use std::fmt;

#[derive(Debug)]
pub enum Error {
    Regex(regex::Error),
    Csv(csv::Error),
    RegexCapturesError,
    TerminalWidthError,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Regex(ref err) => write!(fmt, "{}", err),
            Error::Csv(ref err) => write!(fmt, "{}", err),
            Error::TerminalWidthError => write!(fmt, "Can not get the terminal size."),
            Error::RegexCapturesError => write!(fmt, "Find no match"),
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
