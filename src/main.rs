extern crate colored;
extern crate itertools;
extern crate prettytable;
extern crate regex;
extern crate term_size;
extern crate version_compare;

use colored::{Color, Colorize};
use itertools::EitherOrBoth::Both;
use itertools::Itertools;
use prettytable::{format::consts::FORMAT_CLEAN, Table};
use regex::Regex;
use std::process::Command;
use version_compare::{CompOp, Version, VersionCompare};

fn main() {
    let update_result = {
        let result = Command::new("brew").arg("update").output().unwrap();
        let result_str = String::from_utf8(result.stdout).unwrap_or("".to_owned());

        colorize_update_result(&result_str).unwrap()
    };

    let outdated_result = Command::new("brew").args(&["outdated", "--verbose"]).output().unwrap();
    let formulae = std::str::from_utf8(&outdated_result.stdout).unwrap_or("");

    let output = formulae.lines().map(|formula| {
        let mut splited_formula = formula.split_whitespace();
        let re = Regex::new(r"\((.*)\)").unwrap();
        let (name, current, latest) = (
            splited_formula.next().unwrap_or(""),
            &re.replace(splited_formula.next().unwrap_or(""), "$1"),
            splited_formula.last().unwrap_or(""),
        );
        let current_version = Version::from(current).unwrap();
        let latest_version = Version::from(latest).unwrap();

        let current_version_iter = current_version.parts().iter();
        let mut latest_version_iter = latest_version.parts().iter();

        let index = latest_version_iter
            .clone()
            .zip_longest(current_version_iter)
            .position(|v| match v {
                Both(left, right) => VersionCompare::compare(&left.to_string(), &right.to_string()) != Ok(CompOp::Eq),
                _ => true,
            });

        let colored_latest_version = match index {
            Some(i) => [
                latest_version_iter.by_ref().take(i).join("."),
                latest_version_iter
                    .join(".")
                    .color(match i {
                        0 => Color::Red,
                        1 => Color::Blue,
                        _ => Color::Green,
                    })
                    .to_string(),
            ]
            .iter()
            .filter(|v| v.len() != 0)
            .map(|v| v.to_owned())
            .collect::<Vec<String>>()
            .join("."),
            None => latest_version.to_string(),
        };

        format!("{},{},->,{}", name, current_version, colored_latest_version)
    });

    let update_output = if !formulae.is_empty() {
        let formulae_regex = Regex::new(
            &[
                r"(?m)(^|\s)(".to_owned(),
                formulae
                    .lines()
                    .map(|formula| {
                        let mut splited_formula = formula.split_whitespace();
                        let name = splited_formula.next().unwrap_or("");
                        name.to_owned()
                    })
                    .collect::<Vec<String>>()
                    .join("|"),
                ")(?:  |$)".to_owned(),
            ]
            .iter()
            .filter(|&v| !v.is_empty())
            .map(|v| v.to_owned())
            .collect::<Vec<String>>()
            .join(""),
        )
        .unwrap();

        formulae_regex
            .replace_all(&update_result, "$1\x1b[1m$2\x1b[0m \x1b[32;1m✔\x1b[0m")
            .into_owned()
    } else {
        update_result
    };

    println!("{}", update_output);
    if formulae == "" {
        return;
    }
    println!("{} {}", "==>".blue(), "Oudated Formulae".bold());

    let mut table = Table::from_csv_string(&output.collect::<Vec<String>>().join("\n")).unwrap();
    let mut table_format = *FORMAT_CLEAN;
    table_format.padding(2, 2);
    table.set_format(table_format);
    let trim_regex = Regex::new("(?m)^ *").unwrap();
    print!("{}", trim_regex.replace_all(&table.to_string(), ""));
}

fn colorize_update_result<'a>(target: &'a str) -> Result<String, String> {
    let info_regex = Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)").unwrap();
    let info = info_regex
        .captures_iter(target)
        .map(|info_caps| (&info_caps[0]).to_owned())
        .collect::<Vec<String>>()
        .join("\n");

    let pre_regex = Regex::new(r"==>").unwrap();
    let t = pre_regex.replace_all(target, "\n==>");

    let list_regex = Regex::new(r"(==>) ((?:New|Updated|Renamed|Deleted) Formulae)\n((?:.+\n)+)\n?").unwrap();
    let colored = list_regex
        .captures_iter(&t)
        .map(|list_caps| {
            let formulae_list = build_table((&list_caps[3]).split('\n').collect::<Vec<&str>>());
            format!("{} {}\n{}", &list_caps[1].blue(), &list_caps[2].bold(), formulae_list).to_owned()
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(format!("{}\n{}", info, colored).trim_end_matches('\n').to_owned())
}

fn build_table<'a>(objects: Vec<&'a str>) -> String {
    let gap_size = 2;
    let (terminal_width, _) = term_size::dimensions().unwrap();
    let object_lengths = objects.iter().map(|object| object.len()).collect::<Vec<usize>>();
    let columns = (terminal_width + gap_size) / (object_lengths.iter().max().unwrap_or(&0) + gap_size);

    if objects == vec![""] || columns < 2 {
        return objects.join("\n");
    }

    let rows = (objects.len() + columns - 1) / columns;
    let cols = (objects.len() + rows - 1) / rows;
    let col_width = (terminal_width + gap_size) / cols - gap_size;

    let gap_string = " ".repeat(gap_size);
    let output = (0..rows)
        .map(|row_index| {
            let item_indices_for_now = (row_index..(objects.len() - 1))
                .into_iter()
                .step_by(rows)
                .collect::<Vec<usize>>();
            let item_indices_for_now_len = item_indices_for_now.len();

            let first_n = &item_indices_for_now[..(item_indices_for_now_len - 1)]
                .iter()
                .map(|index| format!("{}{}", objects[*index], " ".repeat(col_width - object_lengths[*index])))
                .collect::<Vec<String>>();
            let last = vec![objects[*item_indices_for_now.iter().last().unwrap()].to_owned()];

            let mut out: Vec<String> = vec![];
            out.extend_from_slice(&first_n);
            out.extend_from_slice(&last);
            format!("{}", out.join(&gap_string))
        })
        .collect::<Vec<String>>()
        .join("\n");

    output
}
