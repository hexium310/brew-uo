extern crate regex;

use regex::Regex;

pub trait BrewUpdate {
    fn parse(&self) -> Result<String, String>;
    fn update_message(&self) -> Result<String, String>;
}

pub trait BrewOutdated {
    fn parse(&self) -> Result<String, String>;
}

#[derive(Clone, Debug)]
pub struct Brew {
    update_result_text: String,
    outdated_result_text: String,
}

impl Brew {
    pub fn new(update_result_text: &str, outdated_result_text: &str) -> Brew {
        Brew {
            update_result_text: update_result_text.to_owned(),
            outdated_result_text: outdated_result_text.to_owned(),
        }
    }
}

impl BrewUpdate for Brew {
    fn parse(&self) -> Result<String, String> {
        let message = self.update_message()?;
        let outdated_list = self
            .outdated_result_text
            .lines()
            .map(|formula| formula.split_whitespace().next().unwrap())
            .collect::<Vec<_>>();
        let list = colorize_updates_list(&self.update_result_text, &outdated_list);

        Ok(format!("{}\n{}", message, list).trim_end_matches('\n').to_owned())
    }

    fn update_message(&self) -> Result<String, String> {
        Ok(
            Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)")
                .unwrap()
                .captures_iter(&self.update_result_text)
                .map(|captures| (&captures[0]).to_owned())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl BrewOutdated for Brew {
    fn parse(&self) -> Result<String, String> {
        Ok("".to_owned())
    }
}
