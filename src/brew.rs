use crate::error::Error;
use crate::terminal::*;
use crate::version::*;
use colored::Colorize;
use prettytable::{format, Table};
use regex::Regex;

pub trait BrewUpdate {
    fn parse(&self) -> Result<String, Error>;
    fn update_message(&self) -> Result<String, Error>;
    fn colorize(&self) -> Result<String, Error>;
    fn build_table(&self, formulae: Vec<&str>, outdated_list: &[&str]) -> String;
}

pub trait BrewOutdated {
    fn parse(&self) -> Result<String, Error>;
    fn outdated_result_csv(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct Brew<T> {
    update_result_text: String,
    outdated_result_text: String,
    terminal: T,
}

impl<T: Terminal> Brew<T> {
    pub fn new(update_result_text: &str, outdated_result_text: &str, terminal: T) -> Brew<T> {
        Brew {
            update_result_text: update_result_text.to_owned(),
            outdated_result_text: outdated_result_text.to_owned(),
            terminal,
        }
    }
}

impl<T: Terminal> BrewUpdate for Brew<T> {
    fn parse(&self) -> Result<String, Error> {
        let message = self.update_message()?;
        let list = self.colorize()?;

        Ok(format!("{}\n{}", message, list).trim_end_matches('\n').to_owned())
    }

    fn update_message(&self) -> Result<String, Error> {
        Ok(
            Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)")?
                .captures_iter(&self.update_result_text)
                .map(|captures| (&captures[0]).to_owned())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    fn colorize(&self) -> Result<String, Error> {
        let outdated_formulae = self
            .outdated_result_text
            .lines()
            .map(|formula| formula.split_whitespace().next().unwrap())
            .collect::<Vec<_>>();

        Ok(
            Regex::new(r"(==>) ((?:New|Updated|Renamed|Deleted) Formulae)\n((?:.+\n)+)\n?")?
                .captures_iter(&self.update_result_text.replace("==>", "\n==>"))
                .map(|captures| {
                    let arrow = &captures[1];
                    let kind = &captures[2];
                    let formulae = &captures[3];
                    let table = self.build_table(formulae.split('\n').collect(), &outdated_formulae);

                    format!("{} {}\n{}", arrow.blue(), kind.bold(), table)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    fn build_table(&self, formulae: Vec<&str>, outdated_formulae: &[&str]) -> String {
        if formulae == vec![""] {
            return "".to_owned();
        }

        let gap_size = 2;
        let gap_string = " ".repeat(gap_size);
        let formulae_length = formulae.len();
        let terminal_width = self.terminal.width().unwrap_or(0);
        let formula_name_lengths = formulae.iter().map(|formula| formula.len()).collect::<Vec<usize>>();
        let column_number = (terminal_width + gap_size) / (formula_name_lengths.iter().max().unwrap_or(&0) + gap_size);

        if column_number < 2 {
            return formulae.join("\n");
        }

        let row_number = (formulae_length + column_number - 1) / column_number;
        let column_width = (terminal_width + gap_size) / ((formulae_length + row_number - 1) / row_number) - gap_size;

        (0..row_number)
            .map(|nth_row| {
                let row_item_indices = (nth_row..(formulae_length - 1)).step_by(row_number);

                row_item_indices
                    .clone()
                    .enumerate()
                    .map(|(index, formula_index)| {
                        let formula_default = formulae[formula_index];
                        let padding = if index != row_item_indices.len() - 1 {
                            " ".repeat(column_width - formula_name_lengths[formula_index])
                        } else {
                            "  ".to_owned()
                        };

                        let (formula, padding_with_checkmark) =
                            if outdated_formulae.iter().any(|&v| v == formula_default) {
                                (
                                    formula_default.bold().to_string(),
                                    padding.replacen("  ", &" ✔".green().to_string(), 1),
                                )
                            } else {
                                (formula_default.to_owned(), padding)
                            };

                        format!("{}{}", formula, padding_with_checkmark)
                    })
                    .collect::<Vec<_>>()
                    .join(&gap_string)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<T: Terminal> BrewOutdated for Brew<T> {
    fn parse(&self) -> Result<String, Error> {
        let outdated_result_csv = self.outdated_result_csv();
        let mut tabulated_outdated_output = Table::from_csv_string(&outdated_result_csv)?;
        let table_format = format::FormatBuilder::new().padding(0, 4).build();
        tabulated_outdated_output.set_format(table_format);

        Ok(tabulated_outdated_output.to_string())
    }

    fn outdated_result_csv(&self) -> String {
        self.outdated_result_text
            .lines()
            .map(|formula| {
                let captures = Regex::new(r"(.+)\s\((.+)\)\s<\s(.+)")
                    .unwrap()
                    .captures(formula)
                    .unwrap();
                let name = &captures[1];
                let current_versions = &captures[2].split(", ").collect::<Vec<_>>();
                let version = Version::new(current_versions, &captures[3]);

                format!(
                    "{},\"{}\",->,{}",
                    name,
                    current_versions.join(", "),
                    version.parse().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
