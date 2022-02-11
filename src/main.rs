#![feature(try_blocks)]

mod brew;
mod color;
mod error;

use std::process::Command;

use colored::Colorize;

use crate::brew::*;
use crate::error::Error;

fn main() {
    if let Err(err) = run_update() {
        println!("command error: {}", err);
    }

    let outdated_result = run_outdated().expect("brew oudated --json failed");
    let outdated = Outdated::new(&outdated_result).unwrap();

    match outdated.format() {
        Ok(output) => {
            println!("{} {}", "==>".blue(), "Oudated Formulae".bold());
            print!("{}", output);
        },
        Err(err) => {
            println!("outdated error: {:?}", err);
        },
    }
}

fn run_outdated() -> Result<String, Error> {
    let result = Command::new("brew").args(&["outdated", "--json"]).output()?;
    Ok(stringify(&result.stdout).to_owned())
}

fn run_update() -> Result<(), Error> {
    try {
        let mut update_process = Command::new("brew").arg("update").spawn()?;
        update_process.wait()?;
    }
}

fn stringify(value: &[u8]) -> &str {
    std::str::from_utf8(value).unwrap_or_default()
}
