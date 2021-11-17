pub mod formatter;
pub mod parser;

pub struct Brew {
    pub outdated: formatter::BrewOutdated,
}

impl Brew {
    pub fn new(outdated_text: &str) -> Self {
        let outdated_data = parser::BrewOutdatedData::new(outdated_text);
        let outdated_formatter = formatter::BrewOutdated::new(&outdated_data);

        Brew {
            outdated: outdated_formatter,
        }
    }
}
