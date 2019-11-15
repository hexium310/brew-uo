use crate::error::Error;

pub trait Terminal {
    fn width(&self) -> Result<usize, Error>;
}

#[derive(Clone, Debug)]
pub struct TerminalInfo {}

impl Terminal for TerminalInfo {
    fn width(&self) -> Result<usize, Error> {
        let (width, _) = term_size::dimensions().ok_or(Error::TerminalWidthError)?;

        Ok(width)
    }
}
