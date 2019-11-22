#![allow(unused_imports)]
use crate::error::Error;
use crate::terminal::*;
use crate::version::*;
use colored::Colorize;
use indoc::indoc;
use itertools::Itertools;
use prettytable::{format, Table};
use regex::Regex;
use std::{
    iter::Zip,
    vec::IntoIter,
};

pub trait BrewUpdate<'a> {
    type Information;

    fn messages(update_result_text: &str) -> Result<Vec<String>, Error>;
    fn information(update_result_text: &'a str) -> Self::Information;

    // fn colorize(&self) -> Result<String, Error>;
    // fn build_table(&self, formulae: Vec<&str>) -> String;
}

pub trait BrewOutdated {
    fn parse(&self) -> Result<String, Error>;
    fn outdated_result_csv(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct BrewData<'a> {
    messages: Vec<String>,
    information: <BrewData<'a> as BrewUpdate<'a>>::Information,
}

impl<'a> BrewData<'a> {
    pub fn _parse(update_result_text: &'a str, _outdated_result_text: &str) -> BrewData<'a> {
        let messages = BrewData::messages(update_result_text).unwrap();
        let information = BrewData::information(update_result_text);

        BrewData {
            messages,
            information,
        }
    }
}

impl<'a> BrewUpdate<'a> for BrewData<'a> {
    type Information = Zip<IntoIter<Vec<&'a str>>, IntoIter<Vec<&'a str>>>;

    fn messages(update_result_text: &str) -> Result<Vec<String>, Error> {
        Ok(
            Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)")?
                .captures_iter(update_result_text)
                .map(|captures| (&captures[0]).to_owned())
                .collect::<Vec<_>>()
        )
    }

    fn information(update_result_text: &'a str) -> Self::Information {
        let grouped = update_result_text
            .lines()
            .group_by(|v| v.find("==>").is_none())
            .into_iter()
            .map(|(k, v)| (k, v.collect::<Vec<_>>()))
            .enumerate()
            .filter(|&(k, (eq, _))| !(k == 0 && eq))
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        let kinds = grouped.clone().into_iter().filter(|&(k, _)| !k).map(|(_, v)| v).collect::<Vec<_>>();
        let formulae = grouped.into_iter().filter(|&(k, _)| k).map(|(_, v)| v).collect::<Vec<_>>();

        kinds.into_iter().zip(formulae.into_iter())
    }

    // fn colorize(&self) -> Result<String, Error> {
    //     Ok(
    //         Regex::new(r"(?P<arrow>==>) (?P<kind>(?:New|Updated|Renamed|Deleted) Formulae)\n(?P<formulae>(?:.+\n)+)\n?")?
    //             .captures_iter(&self.update_result_text.replace("==>", "\n==>"))
    //             .map(|captures| {
    //                 let table = self.build_table((&captures["formulae"]).split('\n').collect());
    //
    //                 format!("{} {}\n{}", &captures["arrow"].blue(), &captures["kind"].bold(), table)
    //             })
    //             .collect::<Vec<_>>()
    //             .join("\n"),
    //     )
    // }
    //
    // fn build_table(&self, formulae: Vec<&str>) -> String {
    //     if formulae == vec![""] {
    //         return "".to_owned();
    //     }
    //
    //     let outdated_formulae = self
    //         .outdated_result_text
    //         .lines()
    //         .map(|formula| formula.split_whitespace().next().unwrap())
    //         .collect::<Vec<_>>();
    //
    //     let gap_size = 2;
    //     let gap_string = " ".repeat(gap_size);
    //
    //     let formulae_length = formulae.len();
    //     let terminal_width = self.terminal.width().unwrap_or(0);
    //     let formula_name_lengths = formulae.iter().map(|formula| formula.len()).collect::<Vec<usize>>();
    //     let column_number = (terminal_width + gap_size) / (formula_name_lengths.iter().max().unwrap_or(&0) + gap_size);
    //
    //     if column_number < 2 {
    //         return formulae.join("\n");
    //     }
    //
    //     let row_number = (formulae_length + column_number - 1) / column_number;
    //     let column_width = (terminal_width + gap_size) / ((formulae_length + row_number - 1) / row_number) - gap_size;
    //
    //     (0..row_number)
    //         .map(|nth_row| {
    //             let row_item_indices = (nth_row..(formulae_length - 1)).step_by(row_number);
    //
    //             row_item_indices
    //                 .clone()
    //                 .enumerate()
    //                 .map(|(index, formula_index)| {
    //                     let formula_default = formulae[formula_index];
    //                     let padding = if index != row_item_indices.len() - 1 {
    //                         " ".repeat(column_width - formula_name_lengths[formula_index])
    //                     } else {
    //                         "  ".to_owned()
    //                     };
    //
    //                     let (formula, padding_with_checkmark) =
    //                         if outdated_formulae.iter().any(|&v| v == formula_default) {
    //                             (
    //                                 formula_default.bold().to_string(),
    //                                 padding.replacen("  ", &" ✔".green().to_string(), 1),
    //                             )
    //                         } else {
    //                             (formula_default.to_owned(), padding)
    //                         };
    //
    //                     format!("{}{}", formula, padding_with_checkmark)
    //                 })
    //                 .collect::<Vec<_>>()
    //                 .join(&gap_string)
    //         })
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // }
}

// impl BrewOutdated for BrewData {
//     fn parse(&self) -> Result<String, Error> {
//         let outdated_result_csv = self.outdated_result_csv();
//         let mut tabulated_outdated_output = Table::from_csv_string(&outdated_result_csv)?;
//         let table_format = format::FormatBuilder::new().padding(0, 4).build();
//         tabulated_outdated_output.set_format(table_format);
//
//         Ok(tabulated_outdated_output.to_string())
//     }
//
//     fn outdated_result_csv(&self) -> String {
//         self.outdated_result_text
//             .lines()
//             .map(|formula| {
//                 let captures = Regex::new(r"(?P<name>.+)\s\((?P<current_versions>.+)\)\s<\s(?P<latest_version>.+)")
//                     .unwrap()
//                     .captures(formula)
//                     .unwrap();
//                 let current_versions = &captures["current_versions"].split(", ").collect::<Vec<_>>();
//                 let version = Version::new(current_versions, &captures["latest_version"]);
//
//                 format!(
//                     "{},\"{}\",->,{}",
//                     &captures["name"],
//                     current_versions.join(", "),
//                     version.parse().unwrap()
//                 )
//             })
//             .collect::<Vec<_>>()
//             .join("\n")
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // const UPDATE_RESUT_TEXT: &str = indoc!(
    //     r#"
    //         Updated 1 tap (homebrew/core).
    //         ==> Updated Formulae
    //         cargo-completion
    //         di
    //         django-completion
    //         eprover
    //         fluid-synth
    //         gdcm
    //         gmic
    //         hypre
    //         zsh
    //         ==> Deleted Formulae
    //         libpagemaker
    //         mypy
    //         oauth2l
    //     "#
    // );

    #[test]
    fn messages() {
        let text = indoc!(
            r#"
                Updated 1 tap (homebrew/core).
                Already up-to-date.
                No changes to formulae.
                ==> Updated Formulae
                typescript
                php
            "#
        );
        let messages = BrewData::messages(text);

        assert_eq!(
            messages.ok(),
            Some(vec![
                "Updated 1 tap (homebrew/core).".to_owned(),
                "Already up-to-date.".to_owned(),
                "No changes to formulae.".to_owned(),
            ])
        );
        assert_eq!(BrewData::messages("").ok(), Some(vec![]));
    }
}
