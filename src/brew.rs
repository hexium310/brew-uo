pub mod formatter;
pub mod parser;

use crate::terminal::*;

pub struct Brew<T>
where
    T: Terminal,
{
    pub update: formatter::BrewUpdate<T>,
    pub outdated: formatter::BrewOutdated,
}

impl<T> Brew<T>
where
    T: Terminal,
{
    pub fn new(update_text: &str, outdated_text: &str, terminal: T) -> Self {
        let update_data = parser::BrewUpdateData::new(update_text);
        let outdated_data = parser::BrewOutdatedData::new(outdated_text);
        let update_formatter = formatter::BrewUpdate::new(&update_data, &outdated_data, terminal);
        let outdated_formatter = formatter::BrewOutdated::new(&outdated_data);

        Brew {
            update: update_formatter,
            outdated: outdated_formatter,
        }
    }
}
