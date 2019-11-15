use crate::brew::parser::*;
use crate::error::Error;
use itertools::Itertools;
use prettytable::{format, Table};

#[derive(Clone, Debug)]
pub struct BrewOutdated {
    data: BrewOutdatedData,
}

impl BrewOutdated {
    pub fn new(data: &BrewOutdatedData) -> Self {
        BrewOutdated { data: data.to_owned() }
    }
}

impl BrewOutdated {
    fn csv(&self) -> String {
        self.data
            .items()
            .map(|formula| {
                let colored = formula.colorize();

                format!(
                    "{},\"{}\",->,{}",
                    colored.name,
                    colored.current_versions.join(", "),
                    colored.latest_version,
                )
            })
            .join("\n")
    }

    pub fn format(&self) -> Result<String, Error> {
        let mut table = Table::from_csv_string(&self.csv())?;
        let table_format = format::FormatBuilder::new().padding(0, 4).build();
        table.set_format(table_format);

        Ok(table.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv() {
        let data = BrewOutdatedData::new("rust (1.38.0, 1.39.0) < 1.40.0");
        let outdated = BrewOutdated::new(&data);

        assert_eq!(outdated.csv(), "rust,\"1.38.0, 1.39.0\",->,1.\u{1b}[34m40.0\u{1b}[0m",)
    }
}
