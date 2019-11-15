use crate::brew::parser::*;
use crate::error::Error;
use crate::terminal::*;
use colored::Colorize;
use itertools::Itertools;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct BrewUpdate<T>
where
    T: Terminal,
{
    outdated_data: BrewOutdatedData,
    update_data: BrewUpdateData,
    terminal: T,
}

impl<T> BrewUpdate<T>
where
    T: Terminal,
{
    pub fn new(update_data: &BrewUpdateData, outdated_data: &BrewOutdatedData, terminal: T) -> Self {
        BrewUpdate {
            outdated_data: outdated_data.to_owned(),
            update_data: update_data.to_owned(),
            terminal,
        }
    }
}

impl<T> BrewUpdate<T>
where
    T: Terminal,
{
    pub fn format(&self) -> Result<String, Error> {
        let update_data = &self.update_data;
        let items = update_data
            .items()
            .map(|(kinds, formulae)| {
                let table = self.tabulate(&formulae);
                let kind = kinds
                    .iter()
                    .map(|kind| Self::colorize_kind(&kind).unwrap_or_else(|_| kind.to_string()))
                    .join("\n");

                format!("{}\n{}", kind, table)
            })
            .join("\n");
        let messages = update_data.messages()?.join("\n");

        if items.is_empty() {
            return Ok(messages);
        }

        if messages.is_empty() {
            return Ok(items);
        }

        Ok(format!("{}\n{}", messages, items))
    }

    fn colorize_kind(kind: &str) -> Result<String, Error> {
        let captures = Regex::new(r"(?P<arrow>==>) (?P<value>.+)")
            .unwrap()
            .captures(kind)
            .ok_or(Error::NoCapturesError)?;

        Ok(format!("{} {}", &captures["arrow"].blue(), &captures["value"].bold()))
    }

    fn tabulate<I, S>(&self, f: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let formulae = f.into_iter().map(|v| v.as_ref().to_owned()).collect::<Vec<_>>();

        if formulae == ["".to_owned()] {
            return "".to_owned();
        }

        let gap_size = 2;
        let gap_string = " ".repeat(gap_size);

        let formulae_length = formulae.len();
        let terminal_width = self.terminal.width().unwrap_or(0);
        let formula_name_lengths = formulae.iter().map(String::len).collect::<Vec<usize>>();
        let column_number = (terminal_width + gap_size) / (formula_name_lengths.iter().max().unwrap_or(&0) + gap_size);

        if column_number < 2 {
            return formulae.join("\n");
        }

        let row_number = (formulae_length + column_number - 1) / column_number;
        let column_width = (terminal_width + gap_size) / ((formulae_length + row_number - 1) / row_number) - gap_size;

        (0..row_number)
            .map(|nth_row| {
                let row_item_indices = (nth_row..(formulae_length - 1)).step_by(row_number);
                let row_item_number = row_item_indices.len();

                row_item_indices
                    .enumerate()
                    .map(|(index, formula_index)| {
                        let formula_default = &formulae[formula_index];
                        let padding = if index != row_item_number - 1 {
                            " ".repeat(column_width - formula_name_lengths[formula_index])
                        } else {
                            "  ".to_owned()
                        };

                        let (formula, padding_with_checkmark) =
                            if self.outdated_data.names().iter().any(|v| v == formula_default) {
                                (
                                    formula_default.bold().to_string(),
                                    padding.replacen("  ", &" ✔".green().to_string(), 1),
                                )
                            } else {
                                (formula_default.to_owned(), padding)
                            };

                        format!("{}{}", formula, padding_with_checkmark)
                    })
                    .join(&gap_string)
            })
            .join("\n")
    }
}

#[cfg(test)]
mod tests {}
