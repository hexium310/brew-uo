use std::slice::SliceIndex;

use anyhow::{Context, Result};
use colored::Colorize;
use itertools::EitherOrBoth::Both;
use itertools::Itertools;
use version_compare::{compare_to, Cmp, Part, Version};

use crate::color::VERSION_COLOR;

#[derive(Clone, Debug)]
pub struct VersionComparison {
    pub installed_version: String,
    pub current_version: String,
    pub delimiters: Vec<String>,
    pub current_version_parts: Vec<String>,
}

impl VersionComparison {
    pub fn new(
        installed_version: &str,
        current_version: &str,
    ) -> VersionComparison {
        let (current_version_parts, delimiters) = VersionComparison::get_delimiters(current_version);

        VersionComparison {
            installed_version: installed_version.to_owned(),
            current_version: current_version.to_owned(),
            delimiters,
            current_version_parts,
        }
    }

    pub fn colorize(&self) -> Result<String> {
        let current_version = match Version::from(&self.current_version) {
            Some(current_version) => current_version,
            None => return Ok(self.current_version.color(VERSION_COLOR.major).to_string()),
        };

        let current_version_parts = current_version.parts();

        let position = match self.get_diff_position(current_version_parts) {
            Some(position) => position,
            None => return Ok(current_version.to_string()),
        };

        let (current_version_parts_without_change, between_delimiter) = match position.checked_sub(1) {
            Some(delimiters_position) => (
                self.build_version(..position, ..delimiters_position)?,
                self.delimiters[delimiters_position].to_owned(),
            ),
            None => ("".to_owned(), "".to_owned()),
        };
        let current_version_parts_with_change = self
            .build_version(position.., position..)?
            .color(match position {
                0 => VERSION_COLOR.major,
                1 => VERSION_COLOR.minor,
                _ => VERSION_COLOR.other,
            });

        Ok(format!(
            "{}{}{}",
            current_version_parts_without_change, between_delimiter, current_version_parts_with_change,
        ))
    }

    fn get_delimiters(version_str: &str) -> (Vec<String>, Vec<String>) {
        Version::from(version_str).map_or_else(
            || (vec![], vec![]),
            |version| {
                let mut delimiters = Vec::new();
                let mut parts = Vec::new();

                for part in version.parts().iter() {
                    let backward = parts.clone().into_iter().interleave(delimiters.clone()).join("");
                    let current_and_forward = version_str.strip_prefix(&backward).unwrap_or_default();
                    let part = current_and_forward.find(|c| c != '0').map_or_else(
                        || part.to_string(),
                        |position| {
                            let times = if position == current_and_forward.find(|c: char| !c.is_ascii_digit()).unwrap_or(position) {
                                0
                            } else {
                                position
                            };
                            "0".repeat(times) + &part.to_string()
                        },
                    );

                    parts.push(part.clone());

                    if let Some(delimiter) = current_and_forward.chars().nth(part.len()) {
                        let delimiter = if delimiter.is_ascii_alphabetic() {
                            "".to_owned()
                        } else {
                            delimiter.to_string()
                        };
                        delimiters.push(delimiter);
                    }
                }
                (parts, delimiters)
            },
        )
    }

    fn get_diff_position(&self, current_version_parts: &[Part]) -> Option<usize> {
        Version::from(&self.installed_version).and_then(|newest_current_version| {
            current_version_parts
                .iter()
                .zip_longest(newest_current_version.parts().iter())
                .position(|v| match v {
                    Both(left, right) => {
                        compare_to(&left.to_string(), &right.to_string(), Cmp::Ne).unwrap_or_else(|_| left != right)
                    },
                    _ => true,
                })
        })
    }

    fn build_version<I>(&self, version_range: I, delimiter_range: I) -> Result<String>
    where
        I: SliceIndex<[String], Output = [String]>,
    {
        let version_parts = self
            .current_version_parts
            .get(version_range)
            .with_context(|| "Index out of range")?
            .iter();
        let delimiters = self
            .delimiters
            .get(delimiter_range)
            .with_context(|| "Index out of range")?;

        Ok(version_parts.interleave(delimiters).join(""))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn before<'a>(
        installed_version: &'a str,
        current_version: &'a str,
    ) -> (VersionComparison, Vec<Part<'a>>) {
        let version = VersionComparison::new(installed_version, current_version);
        let parts = Version::from(current_version).unwrap();
        let parts = parts.parts();

        (version, parts.to_owned())
    }

    #[test]
    fn generate_delimiters_should_return_delimiters_list() {
        assert_eq!(
            VersionComparison::get_delimiters("1.2_3-4"),
            (
                vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "4".to_owned()],
                vec![".".to_owned(), "_".to_owned(), "-".to_owned()],
            )
        );
        assert_eq!(
            VersionComparison::get_delimiters("0001.0002a.0"),
            (
                vec!["0001".to_owned(), "0002".to_owned(), "a".to_owned(), "0".to_owned()],
                vec![".".to_owned(), "".to_owned(), ".".to_owned()],
            )
        );
    }

    #[test]
    fn find_different_part_position_should_return_position() {
        let (version, ref parts) = before("2.0", "1.0");
        assert_eq!(version.get_diff_position(parts), Some(0));

        let (version, ref parts) = before("1.0a", "1.0b");
        assert_eq!(version.get_diff_position(parts), Some(2));

        let (version, ref parts) = before("9d", "9e");
        assert_eq!(version.get_diff_position(parts), Some(1));

        let (version, ref parts) = before("1.0", "1.0");
        assert_eq!(version.get_diff_position(parts), None);
    }

    #[test]
    fn build_version_should_return_string() {
        let (version, ..) = before("", "1.0.0_1");

        assert_eq!(version.build_version(..1, ..0).ok(), Some("1".to_owned()));
        assert_eq!(version.build_version(..2, ..1).ok(), Some("1.0".to_owned()));
        assert_eq!(version.build_version(..3, ..2).ok(), Some("1.0.0".to_owned()));
        assert_eq!(version.build_version(..4, ..3).ok(), Some("1.0.0_1".to_owned()));

        assert_eq!(version.build_version(0.., 0..).ok(), Some("1.0.0_1".to_owned()));
        assert_eq!(version.build_version(1.., 1..).ok(), Some("0.0_1".to_owned()));
        assert_eq!(version.build_version(2.., 2..).ok(), Some("0_1".to_owned()));
        assert_eq!(version.build_version(3.., 3..).ok(), Some("1".to_owned()));
    }

    #[test]
    fn colorize_should_return_version_colored() {
        assert_eq!(
            VersionComparison::new("1.0.0", "2.0.0").colorize().unwrap(),
            format!("{}{}", "", "2.0.0".color(VERSION_COLOR.major))
        );
        assert_eq!(
            VersionComparison::new("1.0.0", "1.1.0").colorize().unwrap(),
            format!("{}{}", "1.", "1.0".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            VersionComparison::new("1.0.0", "1.0.1").colorize().unwrap(),
            format!("{}{}", "1.0.", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("1.0.0_0", "1.0.0_1").colorize().unwrap(),
            format!("{}{}", "1.0.0_", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("1.0.0-0", "1.0.0-1").colorize().unwrap(),
            format!("{}{}", "1.0.0-", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("2.4+20150115", "2.4+20151223_1").colorize().unwrap(),
            format!("{}{}", "2.4+", "20151223_1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("9d", "9e").colorize().unwrap(),
            format!("{}{}", "9", "e".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            VersionComparison::new("3.1", "3.1a").colorize().unwrap(),
            format!("{}{}", "3.1", "a".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("3.1", "3.2a").colorize().unwrap(),
            format!("{}{}", "3.", "2a".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            VersionComparison::new("r2917_1", "r2999").colorize().unwrap(),
            format!("{}{}", "", "r2999".color(VERSION_COLOR.major))
        );
        assert_eq!(
            VersionComparison::new("3.4.1,3041", "3.4.2,3043").colorize().unwrap(),
            format!("{}{}", "3.4.", "2,3043".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("3.1.1", "3.1#2").colorize().unwrap(),
            format!("{}{}", "3.1#", "2".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("0.1.1~git0", "0.1.1~git1").colorize().unwrap(),
            format!("{}{}", "0.1.1~", "git1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new("2021,32.1.0:try2", "2021,32.1.0:try3").colorize().unwrap(),
            format!("{}{}", "2021,32.1.0:", "try3".color(VERSION_COLOR.other))
        );
    }
}
