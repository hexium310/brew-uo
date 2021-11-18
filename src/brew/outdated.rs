use crate::error::Error;
use crate::version::*;
use prettytable::{format, Table};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Outdated {
    pub formulae: Vec<Formula>,
    pub casks: Vec<Formula>,
}

#[derive(Debug, Deserialize)]
pub struct Formula {
    name: String,
    installed_versions: Vec<String>,
    current_version: String,
}

impl Outdated {
    pub(crate) fn new(data: &str) -> serde_json::Result<Self> {
        let outdated: Outdated = serde_json::from_str(data)?;
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
        for Formula { ref name, ref installed_versions, ref current_version } in [&self.formulae, &self.casks].into_iter().flatten()  {
            let current_version = Version::new(installed_versions, current_version).colorize();
            writer.serialize((name, installed_versions, "->", current_version))?;
        }
        writer.flush().unwrap();
        let csv = String::from_utf8(writer.into_inner()?)?;
        Ok(csv)
    }
}
