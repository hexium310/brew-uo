#![allow(unused_imports)]
#![allow(unused_variables)]

#[cfg(test)]
#[macro_use(indoc)]
extern crate indoc;

mod brew;
mod error;
mod terminal;
mod version;

use crate::brew::parser::{
    BrewOutdatedData,
    BrewUpdateData,
};
use crate::terminal::*;
use colored::Colorize;
use std::process::{exit, Command};

fn main() {
    let update_result = run_update();
    let outdated_result = run_outdated();
    BrewUpdateData::new("");
    BrewOutdatedData::new("");

    //
    // let terminal = TerminalInfo {};
    // let brew = BrewData::new(&update_result, &outdated_result, terminal);
    //
    // let update_output = BrewUpdate::output(&brew).unwrap();
    //
    // println!("{}", update_output);
    //
    // if outdated_result.is_empty() {
    //     exit(0);
    // }
    //
    // let outdated_output = BrewOutdated::parse(&brew).unwrap();
    //
    // println!("{} {}", "==>".blue(), "Oudated Formulae".bold());
    // print!("{}", outdated_output);
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
