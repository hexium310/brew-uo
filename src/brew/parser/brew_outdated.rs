use crate::version::*;
use regex::Regex;

pub(crate) trait BrewOutdatedParser {
    type Items: Iterator<Item = BrewOutdatedDetail>;

    fn items(&self) -> Self::Items;
    fn detail(formula: &str) -> Option<BrewOutdatedDetail>;
    fn names(&self) -> Vec<String>;
}

#[derive(Clone, Debug)]
pub struct BrewOutdatedData {
    text: String,
}

impl BrewOutdatedData {
    pub fn new(text: &str) -> Self {
        BrewOutdatedData { text: text.to_owned() }
    }
}

impl BrewOutdatedParser for BrewOutdatedData {
    type Items = impl Iterator<Item = BrewOutdatedDetail>;

    fn detail(formula: &str) -> Option<BrewOutdatedDetail> {
        match Regex::new(r"(?P<name>.+)\s\((?P<current_versions>.+)\)\s<\s(?P<latest_version>.+)")
            .unwrap()
            .captures(formula)
        {
            Some(captures) => Some(BrewOutdatedDetail::new(
                &captures["name"],
                &captures["current_versions"].split(", ").collect::<Vec<_>>(),
                &captures["latest_version"],
            )),
            None => None,
        }
    }

    fn items(&self) -> Self::Items {
        self.text
            .lines()
            .filter_map(Self::detail)
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn names(&self) -> Vec<String> {
        self.items().map(|v| v.name).collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrewOutdatedDetail {
    pub name: String,
    pub current_versions: Vec<String>,
    pub latest_version: String,
}

impl BrewOutdatedDetail {
    pub fn new(name: &str, current_versions: &[&str], latest_version: &str) -> Self {
        BrewOutdatedDetail {
            name: name.to_owned(),
            current_versions: current_versions.iter().map(|&v| v.to_string()).collect::<Vec<_>>(),
            latest_version: latest_version.to_owned(),
        }
    }

    pub fn colorize(&self) -> Self {
        let latest_version = Version::new(&self.current_versions, &self.latest_version)
            .colorize()
            .unwrap();

        BrewOutdatedDetail {
            name: self.name.clone(),
            current_versions: self.current_versions.clone(),
            latest_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail() {
        assert_eq!(
            BrewOutdatedData::detail("rust (1.38.0, 1.39.0) < 1.40.0"),
            Some(BrewOutdatedDetail::new("rust", &["1.38.0", "1.39.0"], "1.40.0"))
        );
        assert_eq!(BrewOutdatedData::detail(""), None);
    }
}
