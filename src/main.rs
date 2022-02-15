#![feature(try_blocks)]

mod brew;
mod color;

use std::process::Command;

use anyhow::Result;
use colored::Colorize;

use crate::brew::*;

fn main() {
    if let Err(err) = run_update() {
        eprintln!("[ERROR] Failed to execute command: {err}");
    }

    let outdated_result = run_outdated().expect("brew oudated --json failed");
    let outdated = Outdated::new(&outdated_result).unwrap();

    if let Some(outdated) = outdated {
        match outdated.format() {
            Ok(output) => {
                println!("{} {}", "==>".blue(), "Oudated Formulae".bold());
                println!("{}", output);
            },
            Err(err) => {
                eprintln!("[ERROR] Failed to build outdated formulae: {err:?}");
            },
        }
    }
}

fn run_outdated() -> Result<String> {
    let result = Command::new("brew").args(&["outdated", "--json"]).output()?;
    Ok(stringify(&result.stdout).to_owned())
}

fn run_update() -> Result<()> {
    try {
        let mut update_process = Command::new("brew").arg("update").spawn()?;
        update_process.wait()?;
    }
}

fn stringify(value: &[u8]) -> &str {
    std::str::from_utf8(value).unwrap_or_default()
}
