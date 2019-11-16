mod brew;
mod terminal;

use colored::{Color, Colorize};
use crate::brew::*;
use crate::terminal::*;
use itertools::EitherOrBoth::{Both, Left};
use itertools::Itertools;
use prettytable::{format, Table};
use regex::Regex;
use std::process::{exit, Command};
use version_compare::{CompOp, Version, VersionCompare};

fn main() {
    let update_result = run_update();
    let outdated_result = run_outdated();
    let terminal = TerminalInfo {};
    let brew = Brew::new(&update_result, &outdated_result, terminal);
    let update_output = BrewUpdate::parse(&brew).unwrap();

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
                        "{}{}{}",
                        latest_version_parts_without_change,
                        delimiters[position - 1],
                        latest_version_parts_with_change,
                    )
                },
            )
        },
    )
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

// #[test]
// #[ignore]
// fn test() {
//     let update = r#"Updated 1 tap (homebrew/core).
//         ==> Updated Formulae
// di
// django-completion
// eprover
// fluid-synth
// gdcm
// gmic
// hypre
// i2p
// imapfilter
// jruby
// paket
// balena-cli
// cfssl
// cmatrix
// drafter
// gnunet
// go
// go@1.12
// godep
// re2
// shared-mime-info
// vagrant-completion
// xcodegen"#;
//
//     let outdated = r#"go (1.13.3) < 1.13.4
// python (3.7.4_1) < 3.7.5
// di (1.1) < 1.2
// re2 (1.1) < 1.2
// godep (1.1) < 1.2
// django-completion (1.1) < 1.2
// eprover (1.1) < 1.2
// fluid-synth (1.1) < 1.2
// gdcm (1.1) < 1.2
// gmic (1.1) < 1.2
// jruby (1.1) < 1.2_1
// vagrant-completion (1.1) < 1.2
// python-yq (2.7.2) < 2.8.1"#;
//
//     println!("{}", build_updates_output(update, outdated));
//     println!("{}", build_outdated_output(outdated));
// }
//
// #[test]
// #[ignore]
// fn test2() {
//     let outdated = r#"go (1.13.3) < 1.13.4
// python (3.7.4_1) < 3.7.5
// di (1.1) < 1.2
// re2 (1.1) < 1.2
// godep (1.1) < 1.2
// django-completion (1.1) < 1.2
// eprover (1.1) < 1.2
// fluid-synth (1.1) < 1.2
// gdcm (1.1) < 1.2
// gmic (1.1) < 1.2
// jruby (1.0, 1.1) < 1.2
// vagrant-completion (1.1) < 1.2
// python-yq (2.7.2) < 2.8.1"#;
//
//     let a = build_outdated_output(outdated);
//     println!("{}", a);
// }
//
// #[test]
// #[ignore]
// fn test3() {
//     let outdated = r#"go (1.13.3) < 1.13.4
// python (3.7.4_1) < 3.7.5
// jruby (1.1) < 1.2_1
// jruby (1.1_1) < 1.2_1
// x264 (r2917) < r2917_1
// x264 (r2917_1) < r2917
// x264 (r2917) < r2917
// python-yq (2.7.2) < 2.8.1"#;
//
//     println!("{}", build_outdated_output(outdated));
// }
//
// #[test]
// fn test4() {
//     let update = r#"di
// django-completion
// eprover"#;
//     let outdated = vec![""];
//
//     struct Mock {};
//     impl Terminal for Mock {
//         fn width(&self) -> Result<usize, String> {
//             Err("Can not get the terminal size.".to_owned())
//         }
//     }
//     let terminal_info = Mock {};
//     println!(
//         "{}",
//         build_updates_list(update.split('\n').collect(), &outdated, &terminal_info)
//     );
// }

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
