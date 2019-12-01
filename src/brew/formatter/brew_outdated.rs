use crate::error::Error;
use crate::terminal::*;
use crate::version::*;
use crate::brew::parser::BrewOutdatedData;
use colored::Colorize;
use indoc::indoc;
use itertools::Itertools;
use prettytable::{format, Table};
use regex::Regex;

trait BrewOutdatedFormatter {
    fn format(&self) -> Result<String, Error>;
    fn csv(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct BrewOutdated {
    data: BrewOutdatedData,
}

impl BrewOutdated {
    pub fn _new(data: BrewOutdatedData) -> Self {
        BrewOutdated {
            data,
        }
    }
}

impl BrewOutdatedFormatter for BrewOutdated {
    fn csv(&self) -> String {
        self.data.information
            .iter()
            .map(|formula| {
                let colored = formula.colorize();

                format!(
                    "{},\"{}\",->,{}",
                    colored.name,
                    colored.current_versions.join(", "),
                    colored.latest_version,
                )
            })
            .join("\n")
    }

    fn format(&self) -> Result<String, Error> {
        Ok("".to_owned())
    }
}

// fn outdated_result_csv(&self) -> String {
//     self.outdated_result_text
//         .lines()
//         .map(|formula| {
//             let captures = Regex::new(r"(?P<name>.+)\s\((?P<current_versions>.+)\)\s<\s(?P<latest_version>.+)")
//                 .unwrap()
//                 .captures(formula)
//                 .unwrap();
//             let current_versions = &captures["current_versions"].split(", ").collect::<Vec<_>>();
//             let version = Version::new(current_versions, &captures["latest_version"]);
//
//             format!(
//                 "{},\"{}\",->,{}",
//                 &captures["name"],
//                 current_versions.join(", "),
//                 version.parse().unwrap()
//             )
//         })
//         .collect::<Vec<_>>()
//         .join("\n")
// }

// fn parse(&self) -> Result<String, Error> {
//     let outdated_result_csv = self.outdated_result_csv();
//     let mut tabulated_outdated_output = Table::from_csv_string(&outdated_result_csv)?;
//     let table_format = format::FormatBuilder::new().padding(0, 4).build();
//     tabulated_outdated_output.set_format(table_format);
//
//     Ok(tabulated_outdated_output.to_string())
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv() {
        let data = BrewOutdatedData::new("rust (1.38.0, 1.39.0) < 1.40.0");
        let outdated = BrewOutdated::_new(data);

        assert_eq!(
            outdated.csv(),
            "rust,\"1.38.0, 1.39.0\",->,1.\u{1b}[34m40.0\u{1b}[0m",
        )
    }
}
