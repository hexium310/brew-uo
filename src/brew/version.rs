use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;

use crate::color::VERSION_COLOR;

#[derive(Clone, Debug)]
pub struct Version {
    pub version: Vec<String>,
}

impl Version {
    pub fn new(version: &str) -> Self {
        let (version, ..) = Self::split(version);

        Self { version }
    }

    fn split(version_str: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut delimiters: Vec<String> = Vec::new();
        let mut parts: Vec<String> = Vec::new();
        let mut version: Vec<String> = Vec::new();
        let groups = version_str.chars().group_by(char::is_ascii_alphanumeric);

        for (key, group) in &groups {
            let part: String = group.into_iter().collect();

            if key {
                parts.push(part.clone());
            } else {
                delimiters.push(part.clone());
            }

            version.push(part);
        }

        (version, parts, delimiters)
    }
}

#[derive(Clone, Debug)]
pub struct VersionComparison {
    pub installed_version: Version,
    pub current_version: Version,
}

impl VersionComparison {
    pub fn new(installed_version: &str, current_version: &str) -> Self {
        let installed_version = Version::new(installed_version);
        let current_version = Version::new(current_version);

        Self {
            installed_version,
            current_version,
        }
    }

    pub fn colorize(&self) -> Result<String> {
        let current_version = &self.current_version.version;

        let position = match self.get_diff_position() {
            Some(position) => position,
            None => return Ok(current_version.join("").color(VERSION_COLOR.major).to_string()),
        };

        let backward = current_version[..position].join("");
        let forward = current_version[position..]
            .join("")
            .color(match (position + 1) / 2 {
                0 => VERSION_COLOR.major,
                1 => VERSION_COLOR.minor,
                _ => VERSION_COLOR.other,
            });

        Ok(format!("{}{}", backward, forward))
    }

    fn get_diff_position(&self) -> Option<usize> {
        let diff = match itertools::diff_with(
            &self.installed_version.version,
            &self.current_version.version,
            |installed_version, current_version| installed_version == current_version
        ) {
            Some(diff) => diff,
            None => return None,
        };

        let (position, ..) = match diff {
            itertools::Diff::FirstMismatch(position, _, current_version_rest) => (position, Some(current_version_rest)),
            itertools::Diff::Shorter(position, _) => (position, None),
            itertools::Diff::Longer(position, current_version_rest) => (position, Some(current_version_rest)),
        };

        Some(position)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn before<'a>(installed_version: &'a str, current_version: &'a str) -> VersionComparison {
        VersionComparison::new(installed_version, current_version)
    }

    #[test]
    fn split_should_return_tuple_of_version_parts_list_and_delimiters_list() {
        assert_eq!(
            Version::split("1.2_3-4"),
            (
                ["1", ".", "2", "_", "3", "-", "4"].into_iter().map(Into::into).collect(),
                ["1", "2", "3", "4"].into_iter().map(Into::into).collect(),
                [".", "_", "-"].into_iter().map(Into::into).collect(),
            ),
        );
        assert_eq!(
            Version::split("0001.0002a.0"),
            (
                ["0001", ".", "0002a", ".", "0"].into_iter().map(Into::into).collect(),
                ["0001", "0002a", "0"].into_iter().map(Into::into).collect(),
                [".", "."].into_iter().map(Into::into).collect(),
            ),
        );
    }

    #[test]
    fn get_diff_position_should_return_position() {
        assert_eq!(before("2.0", "1.0").get_diff_position(), Some(0));
        assert_eq!(before("1", "1.1").get_diff_position(), Some(1));
        assert_eq!(before("1.0a", "1.0b").get_diff_position(), Some(2));
        assert_eq!(before("1.0", "1.0.1").get_diff_position(), Some(3));
        assert_eq!(before("9d", "9e").get_diff_position(), Some(0));
        assert_eq!(before("1.0", "1.0").get_diff_position(), None);
    }

    #[test]
    fn colorize_should_return_version_colored() {
        assert_eq!(
            before("1.0.0", "2.0.0").colorize().unwrap(),
            format!("{}{}", "", "2.0.0".color(VERSION_COLOR.major))
        );
        assert_eq!(
            before("1.0.0", "1.1.0").colorize().unwrap(),
            format!("{}{}", "1.", "1.0".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            before("1.0.0", "1.0.1").colorize().unwrap(),
            format!("{}{}", "1.0.", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("1.0.0_0", "1.0.0_1").colorize().unwrap(),
            format!("{}{}", "1.0.0_", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("1.0.0-0", "1.0.0-1").colorize().unwrap(),
            format!("{}{}", "1.0.0-", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("1.0", "1.0.1").colorize().unwrap(),
            format!("{}{}", "1.0", ".1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("2.4+20150115", "2.4+20151223_1").colorize().unwrap(),
            format!("{}{}", "2.4+", "20151223_1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("9d", "9e").colorize().unwrap(),
            format!("{}{}", "", "9e".color(VERSION_COLOR.major))
        );
        assert_eq!(
            before("3.1", "3.1a").colorize().unwrap(),
            format!("{}{}", "3.", "1a".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            before("3.1", "3.2a").colorize().unwrap(),
            format!("{}{}", "3.", "2a".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            before("r2917_1", "r2999").colorize().unwrap(),
            format!("{}{}", "", "r2999".color(VERSION_COLOR.major))
        );
        assert_eq!(
            before("3.4.1,3041", "3.4.2,3043").colorize().unwrap(),
            format!("{}{}", "3.4.", "2,3043".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("3.1.1", "3.1#2").colorize().unwrap(),
            format!("{}{}", "3.1", "#2".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("0.1.1~git0", "0.1.1~git1").colorize().unwrap(),
            format!("{}{}", "0.1.1~", "git1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            before("2021,32.1.0:try2", "2021,32.1.0:try3").colorize().unwrap(),
            format!("{}{}", "2021,32.1.0:", "try3".color(VERSION_COLOR.other))
        );
    }
}
