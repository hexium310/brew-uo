extern crate colored;
extern crate regex;
extern crate version_compare;
extern crate itertools;
extern crate prettytable;

use std::process::Command;
use colored::*;
use regex::Regex;
use version_compare::{Version, VersionCompare, CompOp};
use itertools::Itertools;
use itertools::EitherOrBoth::*;
use prettytable::{Table, format::consts::FORMAT_CLEAN};

fn main() {
    let update_result = Command::new("brew").arg("update").output().unwrap();
    print!("{}", std::str::from_utf8(&update_result.stdout).unwrap_or(""));
    let outdated_result = Command::new("brew")
        .args(&["outdated", "--verbose"])
        .output()
        .unwrap();
    let formulae = std::str::from_utf8(&outdated_result.stdout).unwrap_or("");

    if formulae == "" {
        return;
    }

    let output = formulae.lines().map(|formula| {
        let mut splited_formula = formula.split_whitespace();
        let re = Regex::new(r"\((.*)\)").unwrap();
        let (
            name,
            current,
            latest
        ) = (
            splited_formula.next().unwrap_or(""),
            &re.replace(splited_formula.next().unwrap_or(""), "$1"),
            splited_formula.last().unwrap_or("")
        );
        let current_version = Version::from(current).unwrap();
        let latest_version = Version::from(latest).unwrap();

        let current_version_iter = current_version.parts().iter();
        let mut latest_version_iter = latest_version.parts().iter();

        let index = latest_version_iter
            .clone()
            .zip_longest(current_version_iter)
            .position(|v| {
                match v {
                    Both(left, right) =>
                        VersionCompare::compare(&left.to_string(), &right.to_string()) != Ok(CompOp::Eq),
                    _ => true,
                }
            });

        let colored_latest_version = match index {
            Some(i) => [
                latest_version_iter.by_ref().take(i).join("."),
                latest_version_iter.join(".").color(
                    match i {
                        0 => "red",
                        1 => "blue",
                        _ => "green",
                    }
                ).to_string()
            ].iter().filter(|v| v.len() != 0).map(|v| v.to_owned()).collect::<Vec<String>>().join("."),
            None => latest_version.to_string()
        };

        format!("{},{},->,{}", name, current_version, colored_latest_version)
    });

    println!("{} {}", "==>".blue(), "Oudated Formulae".bold());

    let mut table = Table::from_csv_string(&output.collect::<Vec<String>>().join("\n")).unwrap();
    let mut table_format = *FORMAT_CLEAN;
    table_format.padding(2, 2);
    table.set_format(table_format);
    table.printstd();
}
