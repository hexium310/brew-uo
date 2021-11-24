use prettytable::{format, Table};
use serde::Deserialize;

use crate::brew::version::*;
use crate::error::Error;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Outdated {
    pub formulae: Vec<Formula>,
    pub casks: Vec<Formula>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Formula {
    name: String,
    installed_versions: Vec<String>,
    current_version: String,
}

impl Outdated {
    pub(crate) fn new(data: &str) -> serde_json::Result<Self> {
        let outdated = serde_json::from_str::<Outdated>(data)?;
        Ok(outdated)
    }

    pub fn format(&self) -> Result<String, Error> {
        let mut table = Table::from_csv_string(&self.to_csv()?)?;
        let table_format = format::FormatBuilder::new().padding(0, 4).build();
        table.set_format(table_format);

        Ok(table.to_string())
    }

    fn to_csv(&self) -> Result<String, Error> {
        let mut writer = csv::Writer::from_writer(vec![]);
        for Formula {
            ref name,
            ref installed_versions,
            ref current_version,
        } in [&self.formulae, &self.casks].into_iter().flatten()
        {
            let current_version = VersionComparison::new(installed_versions, current_version).colorize();
            writer.serialize((name, installed_versions, "->", current_version))?;
        }
        writer.flush().unwrap();
        let csv = String::from_utf8(writer.into_inner()?)?;
        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::VERSION_COLOR;

    #[test]
    fn new_returns_outdated_struct() {
        let data = r#"
            {
              "formulae": [
                {
                  "name": "php",
                  "installed_versions": [
                    "8.0.12"
                  ],
                  "current_version": "8.0.13",
                  "pinned": false,
                  "pinned_version": null
                }
              ],
              "casks": [
                {
                  "name": "powershell",
                  "installed_versions": [
                    "7.1.0"
                  ],
                  "current_version": "7.2.0",
                  "pinned": false,
                  "pinned_version": null
                }
              ]
            }
        "#;
        assert_eq!(
            Outdated::new(data).unwrap(),
            Outdated {
                formulae: vec![Formula {
                    name: "php".to_owned(),
                    installed_versions: vec!["8.0.12".to_owned()],
                    current_version: "8.0.13".to_owned(),
                }],
                casks: vec![Formula {
                    name: "powershell".to_owned(),
                    installed_versions: vec!["7.1.0".to_owned()],
                    current_version: "7.2.0".to_owned(),
                }],
            }
        );

        let data = r#"
            {
              "formulae": [

              ],
              "casks": [

              ]
            }
        "#;
        assert_eq!(
            Outdated::new(data).unwrap(),
            Outdated {
                formulae: vec![],
                casks: vec![],
            }
        );
    }

    #[test]
    fn to_csv_returns_csv_with_color() {
        use colored::Colorize;

        let data = r#"
            {
              "formulae": [
                {
                  "name": "php",
                  "installed_versions": [
                    "8.0.12"
                  ],
                  "current_version": "8.0.13",
                  "pinned": false,
                  "pinned_version": null
                }
              ],
              "casks": [
                {
                  "name": "powershell",
                  "installed_versions": [
                    "7.1.0"
                  ],
                  "current_version": "7.2.0",
                  "pinned": false,
                  "pinned_version": null
                }
              ]
            }
        "#;
        let outdated = Outdated::new(data).unwrap();
        assert_eq!(
            outdated.to_csv().unwrap(),
            format!(
                "php,8.0.12,->,8.0.{}\npowershell,7.1.0,->,7.{}\n",
                "13".color(VERSION_COLOR.other),
                "2.0".color(VERSION_COLOR.minor)
            )
        );
    }

    #[test]
    fn format_returns_tabular_formulae() {
        use colored::Colorize;

        let data = r#"
            {
              "formulae": [
                {
                  "name": "php",
                  "installed_versions": [
                    "8.0.12"
                  ],
                  "current_version": "8.0.13",
                  "pinned": false,
                  "pinned_version": null
                }
              ],
              "casks": [
                {
                  "name": "powershell",
                  "installed_versions": [
                    "7.1.0"
                  ],
                  "current_version": "7.2.0",
                  "pinned": false,
                  "pinned_version": null
                }
              ]
            }
        "#;
        let outdated = Outdated::new(data).unwrap();
        assert_eq!(
            outdated.format().unwrap(),
            format!(
                "{}\n{}\n",
                format!(
                    "php           8.0.12    ->    8.0.{}    ",
                    "13".color(VERSION_COLOR.other)
                ),
                format!(
                    "powershell    7.1.0     ->    7.{  }    ",
                    "2.0".color(VERSION_COLOR.minor)
                )
            )
        );
    }
}
