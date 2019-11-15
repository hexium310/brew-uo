use std::fmt;

#[derive(Debug)]
pub enum Error {
    Csv(csv::Error),
    Command(std::io::Error),
    NoCapturesError,
    TerminalWidthError,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Csv(ref err) => write!(fmt, "{}", err),
            Error::Command(ref err) => write!(fmt, "{}", err),
            Error::NoCapturesError => write!(fmt, "Can not capture groups."),
            Error::TerminalWidthError => write!(fmt, "Can not get the terminal size."),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Command(err)
    }
}
