use anyhow::{Context, Result};
use itertools::Itertools;
use prettytable::{format, Table};
use serde::Deserialize;

use crate::brew::version::*;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Json {
    pub formulae: Vec<Formula>,
    pub casks: Vec<Cask>,
}

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

#[derive(Debug, Deserialize, PartialEq)]
pub struct Cask {
    name: String,
    installed_versions: String,
    current_version: String,
}

impl From<Cask> for Formula {
    fn from(c: Cask) -> Self {
        Formula {
            name: c.name,
            installed_versions: c.installed_versions.split(", ").map_into().collect(),
            current_version: c.current_version,
        }
    }
}

impl Outdated {
    pub(crate) fn new(data: &str) -> Result<Option<Self>> {
        let Json { formulae, casks } = serde_json::from_str(data).with_context(|| format!("Invalid JSON:\n{data}"))?;
        let casks = casks.into_iter().map_into().collect_vec();
        if formulae.is_empty() && casks.is_empty() {
            return Ok(None);
        }
        let outdated = Outdated {
            formulae,
            casks,
        };

        Ok(Some(outdated))
    }

    pub fn format(&self) -> Result<String> {
        let csv = self.to_csv().with_context(|| "Failed to make a CSV")?;
        let mut table = Table::from_csv_string(&csv).with_context(|| format!("Failed to create a table:\n{csv}"))?;
        let table_format = format::FormatBuilder::new().padding(0, 4).build();
        table.set_format(table_format);

        Ok(table.to_string().lines().map(|v| v.trim_end()).join("\n"))
    }

    fn to_csv(&self) -> Result<String> {
        let mut writer = csv::Writer::from_writer(vec![]);
        for Formula { name, installed_versions, current_version } in itertools::chain(&self.formulae, &self.casks) {
            let latest_installed_version = match installed_versions.last() {
                Some(installed_version) => installed_version,
                None => {
                    eprintln!("[ERROR] There are no installed versions: {name}");
                    continue;
                },
            };
            let version = VersionComparison::new(latest_installed_version, current_version);
            let colorized_current_version = match version.colorize() {
                Ok(colorized) => colorized,
                Err(error) => {
                    eprintln!("[ERROR] Failed to colorize the current version because of \"{error}\": {name}");
                    continue;
                },
            };

            if writer.serialize((name, latest_installed_version, "->", &colorized_current_version)).is_err() {
                eprintln!("[ERROR] Failed to serialize it as CSV: name = {name}, latest installed version = {latest_installed_version}, colorized current version = {colorized_current_version:?}");
                continue;
            }
        }
        let csv = String::from_utf8(writer.into_inner()?)?;
        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use indoc::{formatdoc, indoc};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::color::VERSION_COLOR;

    const DATA: &str = indoc! {r#"
        {
          "formulae": [
            {
              "name": "curl",
              "installed_versions": [
                "7.80.0",
                "7.80.0"
              ],
              "current_version": "7.80.0_1",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "jpeg",
              "installed_versions": [
                "9d"
              ],
              "current_version": "9e",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "php",
              "installed_versions": [
                "8.0.12"
              ],
              "current_version": "8.0.13",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "picat",
              "installed_versions": [
                "3.1.1"
              ],
              "current_version": "3.1#2",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "srmio",
              "installed_versions": [
                "0.1.0"
              ],
              "current_version": "0.1.1~git1",
              "pinned": false,
              "pinned_version": null
            }
          ],
          "casks": [
            {
              "name": "atok",
              "installed_versions": "2021,32.1.0:try2",
              "current_version": "2021,32.1.0:try3",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "duplicati",
              "installed_versions": "2.0.6.1,beta:2021-05-03",
              "current_version": "2.0.6.3,beta:2021-06-17",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "powershell",
              "installed_versions": "7.1.0",
              "current_version": "7.2.0",
              "pinned": false,
              "pinned_version": null
            },
            {
              "name": "sequel-ace",
              "installed_versions": "3.4.0,3038, 3.4.1,3041",
              "current_version": "3.4.2,3043",
              "pinned": false,
              "pinned_version": null
            }
          ]
        }
    "#};

    #[test]
    fn new_should_return_outdated_struct() {
        assert_eq!(
            Outdated::new(DATA).unwrap().unwrap(),
            Outdated {
                formulae: vec![
                    Formula {
                        name: "curl".to_owned(),
                        installed_versions: vec!["7.80.0".to_owned(), "7.80.0".to_owned()],
                        current_version: "7.80.0_1".to_owned(),
                    },
                    Formula {
                        name: "jpeg".to_owned(),
                        installed_versions: vec!["9d".to_owned()],
                        current_version: "9e".to_owned(),
                    },
                    Formula {
                        name: "php".to_owned(),
                        installed_versions: vec!["8.0.12".to_owned()],
                        current_version: "8.0.13".to_owned(),
                    },
                    Formula {
                        name: "picat".to_owned(),
                        installed_versions: vec!["3.1.1".to_owned()],
                        current_version: "3.1#2".to_owned(),
                    },
                    Formula {
                        name: "srmio".to_owned(),
                        installed_versions: vec!["0.1.0".to_owned()],
                        current_version: "0.1.1~git1".to_owned(),
                    },
                ],
                casks: vec![
                    Formula {
                        name: "atok".to_owned(),
                        installed_versions: vec!["2021,32.1.0:try2".to_owned()],
                        current_version: "2021,32.1.0:try3".to_owned(),
                    },
                    Formula {
                        name: "duplicati".to_owned(),
                        installed_versions: vec!["2.0.6.1,beta:2021-05-03".to_owned()],
                        current_version: "2.0.6.3,beta:2021-06-17".to_owned(),
                    },
                    Formula {
                        name: "powershell".to_owned(),
                        installed_versions: vec!["7.1.0".to_owned()],
                        current_version: "7.2.0".to_owned(),
                    },
                    Formula {
                        name: "sequel-ace".to_owned(),
                        installed_versions: vec!["3.4.0,3038".to_owned(), "3.4.1,3041".to_owned()],
                        current_version: "3.4.2,3043".to_owned(),
                    }
                ],
            }
        );

        let data = indoc! {r#"
            {
              "formulae": [

              ],
              "casks": [

              ]
            }
        "#};
        assert!(Outdated::new(data).unwrap().is_none());
    }

    #[test]
    fn to_csv_should_return_csv_with_color() {
        use colored::Colorize;

        let outdated = Outdated::new(DATA).unwrap().unwrap();
        assert_eq!(
            outdated.to_csv().unwrap(),
            formatdoc! {r#"
                curl,7.80.0,->,7.80.0{curl}
                jpeg,9d,->,{jpeg}
                php,8.0.12,->,8.0.{php}
                picat,3.1.1,->,3.1{picat}
                srmio,0.1.0,->,0.1.{srmio}
                atok,"2021,32.1.0:try2",->,"2021,32.1.0:{atok}"
                duplicati,"2.0.6.1,beta:2021-05-03",->,"2.0.6.{duplicati}"
                powershell,7.1.0,->,7.{powershell}
                sequel-ace,"3.4.1,3041",->,"3.4.{sequel_ace}"
                "#,
                curl = "_1".color(VERSION_COLOR.other),
                jpeg = "9e".color(VERSION_COLOR.major),
                php = "13".color(VERSION_COLOR.other),
                picat = "#2".color(VERSION_COLOR.other),
                srmio = "1~git1".color(VERSION_COLOR.other),
                atok = "try3".color(VERSION_COLOR.other),
                duplicati = "3,beta:2021-06-17".color(VERSION_COLOR.other),
                powershell = "2.0".color(VERSION_COLOR.minor),
                sequel_ace = "2,3043".color(VERSION_COLOR.other),
            }
        );
    }

    #[test]
    fn format_should_return_tabular_formulae() {
        use colored::Colorize;

        let outdated = Outdated::new(DATA).unwrap().unwrap();
        assert_eq!(
            outdated.format().unwrap(),
            formatdoc! {"
                curl          7.80.0                     ->    7.80.0{curl}
                jpeg          9d                         ->    {jpeg}
                php           8.0.12                     ->    8.0.{php}
                picat         3.1.1                      ->    3.1{picat}
                srmio         0.1.0                      ->    0.1.{srmio}
                atok          2021,32.1.0:try2           ->    2021,32.1.0:{atok}
                duplicati     2.0.6.1,beta:2021-05-03    ->    2.0.6.{duplicati}
                powershell    7.1.0                      ->    7.{powershell}
                sequel-ace    3.4.1,3041                 ->    3.4.{sequel_ace}",
                curl = "_1".color(VERSION_COLOR.other),
                jpeg = "9e".color(VERSION_COLOR.major),
                php = "13".color(VERSION_COLOR.other),
                picat = "#2".color(VERSION_COLOR.other),
                srmio = "1~git1".color(VERSION_COLOR.other),
                atok = "try3".color(VERSION_COLOR.other),
                duplicati = "3,beta:2021-06-17".color(VERSION_COLOR.other),
                powershell = "2.0".color(VERSION_COLOR.minor),
                sequel_ace = "2,3043".color(VERSION_COLOR.other),
            }
        );
    }
}
