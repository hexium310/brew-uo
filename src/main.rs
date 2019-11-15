#![feature(type_alias_impl_trait)]

#[cfg(test)]
#[macro_use(indoc)]
extern crate indoc;

mod brew;
mod error;
mod range;
mod terminal;
mod version;

use crate::brew::Brew;
use crate::error::Error;
use crate::terminal::*;
use colored::Colorize;
use std::process::{exit, Command};

fn main() {
    let update_result = run_update().unwrap_or_else(|err| {
        println!("command error: {}", err);
        "".to_owned()
    });
    let outdated_result = run_outdated().unwrap_or_else(|err| {
        println!("command error: {}", err);
        "".to_owned()
    });

    let terminal = TerminalInfo {};
    let brew = Brew::new(&update_result, &outdated_result, terminal);

    match brew.update.format() {
        Ok(output) if &output != "" => {
            println!("{}", output);
        },
        Ok(_) => (),
        Err(err) => {
            println!("update error: {:?}", err);
        },
    };

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

fn run_update() -> Result<String, Error> {
    let result = Command::new("brew").arg("update").output()?;
    Ok(stringify(&result.stdout))
}

fn stringify(value: &[u8]) -> String {
    String::from_utf8(value.to_owned()).unwrap_or_else(|_| "".to_owned())
}
