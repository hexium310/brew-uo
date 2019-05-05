extern crate colored;
extern crate regex;

use std::process::Command;
use colored::*;
use regex::Regex;

fn main() {
    Command::new("brew").arg("update").output().unwrap();
    let outdated_result = Command::new("brew")
        .args(&["outdated", "--verbose"])
        .output()
        .unwrap();
    let formulae = std::str::from_utf8(&outdated_result.stdout).unwrap_or("");

    if formulae == "" {
        return;
    }

    let output = formulae.lines().map(|formula| {
        let mut splited_outdated = formula.split_whitespace();
        let re = Regex::new(r"\((.*)\)").unwrap();
        let (
            name,
            latest_version,
            _,
            current_version
        ) = (
            splited_outdated.next().unwrap_or(""),
            re.replace(splited_outdated.next().unwrap_or(""), "$1"),
            splited_outdated.next().unwrap_or(""),
            splited_outdated.next().unwrap_or("")
        );
        format!("{} {} →  {}", name, latest_version, current_version)
    });

    println!("{} {}", "==>".blue(), "Oudated Formulae".bold());
    println!("{}", output.collect::<Vec<String>>().join("\n"));
}
