extern crate colored;
extern crate itertools;
extern crate prettytable;
extern crate regex;
extern crate term_size;
extern crate version_compare;

use colored::{Color, Colorize};
use itertools::EitherOrBoth::{Both, Left};
use itertools::Itertools;
use prettytable::{format, Table};
use regex::Regex;
use std::process::{exit, Command};
use version_compare::{CompOp, Version, VersionCompare};

fn main() {
    let update_result = run_update();
    let outdated_result = run_outdated();
    let update_output = build_updates_output(&update_result, &outdated_result);

    println!("{}", update_output);

    if outdated_result.is_empty() {
        exit(0);
    }

    let outdated_output = build_outdated_output(&outdated_result);

    println!("{} {}", "==>".blue(), "Oudated Formulae".bold());
    print!("{}", outdated_output);
}

fn build_outdated_csv(outdated_result: &str) -> String {
    outdated_result
        .lines()
        .map(|formula| {
            let captures = Regex::new(r"(.+)\s\((.+)\)\s<\s(.+)")
                .unwrap()
                .captures(formula)
                .unwrap();
            let name = &captures[1];
            let current_versions = &captures[2].split(", ").collect::<Vec<_>>();
            let colored_latest_version = colorize_latest_version(current_versions, &captures[3]);

            format!(
                "{},\"{}\",->,{}",
                name,
                current_versions.join(", "),
                colored_latest_version
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_outdated_output(outdated_result: &str) -> String {
    let outdated_result_csv = build_outdated_csv(outdated_result);
    let mut tabulated_outdated_output = Table::from_csv_string(&outdated_result_csv).unwrap();
    let table_format = format::FormatBuilder::new().padding(0, 4).build();
    tabulated_outdated_output.set_format(table_format);

    tabulated_outdated_output.to_string()
}

fn build_updates_output(update_result: &str, outdated_result: &str) -> String {
    let message = extract_update_message(update_result);
    let outdated_list = outdated_result
        .lines()
        .map(|formula| {
            let a = formula.split_whitespace().collect::<Vec<_>>();
            a.first().unwrap().to_owned()
        })
        .collect::<Vec<_>>();
    let list = colorize_updates_list(update_result, &outdated_list);

    format!("{}\n{}", message, list).trim_end_matches('\n').to_owned()
}

fn build_updates_list(formulae: Vec<&str>, outdated_list: &[&str]) -> String {
    if formulae == vec![""] {
        return "".to_owned();
    }

    let gap_size = 2;
    let gap_string = " ".repeat(gap_size);
    let formulae_length = formulae.len();
    let (terminal_width, _) = term_size::dimensions().unwrap();
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

                    let (formula, padding_with_checkmark) = if outdated_list.iter().any(|&v| v == formula_default) {
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

fn build_version(version_parts: &[version_compare::VersionPart], delimiter: &[String]) -> String {
    let zipped = version_parts
        .iter()
        .map(|v| v.to_string())
        .zip_longest(delimiter.iter().map(|v| v.to_owned()));

    zipped
        .clone()
        .fold(
            Vec::with_capacity(zipped.len() * 2) as Vec<String>,
            |mut accumulator, tuple| match tuple {
                Both(part, delimiter) => {
                    accumulator.push(part);
                    accumulator.push(delimiter);
                    accumulator
                },
                Left(part) => {
                    accumulator.push(part);
                    accumulator
                },
                _ => accumulator,
            },
        )
        .join("")
}

fn get_delimiters(version_str: &str) -> Vec<String> {
    let delimiter_chars = ['.', '_'];

    version_str
        .matches(|version_char| {
            delimiter_chars
                .iter()
                .any(|&delimiter_char| delimiter_char == version_char)
        })
        .map(|v| v.to_owned())
        .collect::<Vec<_>>()
}

fn colorize_latest_version(current_versions: &[&str], latest_version_str: &str) -> String {
    let delimiters = get_delimiters(latest_version_str);

    Version::from(latest_version_str).map_or_else(
        || latest_version_str.to_owned(),
        |latest_version| {
            let latest_version_parts = latest_version.parts();
            let different_part_position = find_different_part_position(current_versions, latest_version_parts);

            different_part_position.map_or_else(
                || latest_version.to_string(),
                |position| {
                    let latest_version_parts_without_change =
                        build_version(&latest_version_parts[..position], &delimiters[..position - 1]);
                    let latest_version_parts_with_change =
                        build_version(&latest_version_parts[position..], &delimiters[position..])
                            .color(match position {
                                0 => Color::Red,
                                1 => Color::Blue,
                                _ => Color::Green,
                            })
                            .to_string();

                    format!(
                        "{}.{}",
                        latest_version_parts_without_change, latest_version_parts_with_change
                    )
                },
            )
        },
    )
}

fn colorize_updates_list(command_result: &str, outdated_list: &[&str]) -> String {
    Regex::new(r"(==>) ((?:New|Updated|Renamed|Deleted) Formulae)\n((?:.+\n)+)\n?")
        .unwrap()
        .captures_iter(&command_result.replace("==>", "\n==>"))
        .map(|captures| {
            let list = build_updates_list((&captures[3]).split('\n').collect(), outdated_list);
            format!("{} {}\n{}", &(captures[1]).blue(), &(captures[2]).bold(), list)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_update_message(command_result: &str) -> String {
    Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)")
        .unwrap()
        .captures_iter(command_result)
        .map(|captures| (&captures[0]).to_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_different_part_position(
    current_versions: &[&str],
    latest_version_parts: &[version_compare::VersionPart],
) -> Option<usize> {
    Version::from(current_versions.last().unwrap()).map_or_else(
        || Some(1),
        |newest_current_version| {
            latest_version_parts
                .iter()
                .zip_longest(newest_current_version.parts().iter())
                .position(|v| match v {
                    Both(left, right) => {
                        VersionCompare::compare_to(&left.to_string(), &right.to_string(), &CompOp::Ne).unwrap()
                    },
                    _ => true,
                })
        },
    )
}

fn run_outdated() -> String {
    let result = Command::new("brew").args(&["outdated", "--verbose"]).output().unwrap();
    stringify(result.stdout)
}

fn run_update() -> String {
    let result = Command::new("brew").arg("update").output().unwrap();
    stringify(result.stdout)
}

fn stringify(value: Vec<u8>) -> String {
    String::from_utf8(value).unwrap_or_else(|_| "".to_owned())
}
