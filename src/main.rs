#![feature(type_alias_impl_trait, try_blocks)]

mod brew;
mod error;
mod range;
mod version;

use crate::brew::Brew;
use crate::error::Error;
use colored::Colorize;
use std::process::{exit, Command};

fn main() {
    if let Err(err) = run_update() {
        println!("command error: {}", err);
    }

    let outdated_result = run_outdated().unwrap_or_else(|err| {
        println!("command error: {}", err);
        "".to_owned()
    });

    let brew = Brew::new(&outdated_result);

    if outdated_result.is_empty() {
        exit(0);
    }

    match brew.outdated.format() {
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
    let result = Command::new("brew").args(&["outdated", "--verbose"]).output()?;
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
