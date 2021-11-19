#![feature(try_blocks)]

mod brew;
mod error;
mod range;

use crate::brew::*;
use crate::error::Error;
use colored::Colorize;
use std::process::Command;

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
    Ok(stringify(&result.stdout))
}

fn run_update() -> Result<(), Error> {
    try {
        let mut update_process = Command::new("brew").arg("update").spawn()?;
        update_process.wait()?;
    }
}

fn stringify(value: &[u8]) -> String {
    String::from_utf8(value.to_owned()).unwrap_or_else(|_| "".to_owned())
}
