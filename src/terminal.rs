pub trait Terminal {
    fn width(&self) -> Result<usize, String>;
}

#[derive(Clone, Debug)]
pub struct TerminalInfo {}

impl Terminal for TerminalInfo {
    fn width(&self) -> Result<usize, String> {
        let (width, _) = term_size::dimensions().ok_or_else(|| "Can not get the terminal size.".to_owned())?;

        Ok(width)
    }
}
