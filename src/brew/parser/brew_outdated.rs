use regex::Regex;

trait BrewOutdatedParser {
    fn information(outdated_result_text: &str) -> Vec<BrewOutdatedDetail>;
    fn detail(formula: &str) -> Option<BrewOutdatedDetail>;
}

#[derive(Clone, Debug)]
pub struct BrewOutdatedData {
    information: Vec<BrewOutdatedDetail>,
}

impl BrewOutdatedData {
    pub fn new(outdated_result_text: &str) -> Self {
        let information = Self::information(outdated_result_text);

        BrewOutdatedData { information }
    }
}

impl BrewOutdatedParser for BrewOutdatedData {
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

    fn information(outdated_result_text: &str) -> Vec<BrewOutdatedDetail> {
        outdated_result_text
            .lines()
            .filter_map(Self::detail)
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrewOutdatedDetail {
    name: String,
    current_versions: Vec<String>,
    latest_version: String,
}

impl BrewOutdatedDetail {
    pub fn new(name: &str, current_versions: &[&str], latest_version: &str) -> Self {
        BrewOutdatedDetail {
            name: name.to_owned(),
            current_versions: current_versions.iter().map(|&v| v.to_string()).collect::<Vec<_>>(),
            latest_version: latest_version.to_owned(),
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
