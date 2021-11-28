use std::ops::Bound::*;

use colored::Colorize;
use itertools::EitherOrBoth::Both;
use itertools::Itertools;
use version_compare::{compare_to, Cmp, Part, Version};

use crate::color::VERSION_COLOR;
use crate::error::Error;
use crate::range::*;

#[derive(Clone, Debug)]
pub struct VersionComparison {
    pub latest_installed_version: String,
    pub current_version: String,
    pub delimiters: Vec<String>,
}

impl VersionComparison {
    pub fn new(
        installed_versions: impl IntoIterator<Item = impl AsRef<str>>,
        current_version: &str,
    ) -> VersionComparison {
        let delimiters = VersionComparison::get_delimiters(current_version);
        let latest_installed_version = VersionComparison::get_latest_installed_version(installed_versions);

        VersionComparison {
            latest_installed_version,
            current_version: current_version.to_owned(),
            delimiters,
        }
    }

    pub fn colorize(&self) -> String {
        Version::from(&self.current_version).map_or_else(
            || self.current_version.color(VERSION_COLOR.major).to_string(),
            |current_version| {
                let current_version_parts = current_version.parts();
                self.get_diff_position(current_version_parts).map_or_else(
                    || current_version.to_string(),
                    |position| {
                        let (current_version_parts_without_change, between_delimiter) =
                            position.checked_sub(1).map_or_else(
                                || ("".to_owned(), "".to_owned()),
                                |delimiters_position| {
                                    (
                                        self.build_version(current_version_parts, ..position, ..delimiters_position)
                                            .unwrap(),
                                        self.delimiters[delimiters_position].to_owned(),
                                    )
                                },
                            );
                        let current_version_parts_with_change = self
                            .build_version(current_version_parts, position.., position..)
                            .unwrap()
                            .color(match position {
                                0 => VERSION_COLOR.major,
                                1 => VERSION_COLOR.minor,
                                _ => VERSION_COLOR.other,
                            })
                            .to_string();

                        format!(
                            "{}{}{}",
                            current_version_parts_without_change, between_delimiter, current_version_parts_with_change,
                        )
                    },
                )
            },
        )
    }

    fn get_latest_installed_version(installed_versions: impl IntoIterator<Item = impl AsRef<str>>) -> String {
        let installed_versions = installed_versions
            .into_iter()
            .map(|v| v.as_ref().to_owned())
            .collect::<Vec<String>>();
        installed_versions.last().unwrap().to_owned()
    }

    fn get_delimiters(version_str: &str) -> Vec<String> {
        let delimiter_chars = ['.', '_', '-', '+', ','];

        version_str
            .matches(|version_char| {
                delimiter_chars
                    .iter()
                    .any(|&delimiter_char| delimiter_char == version_char)
            })
            .map(|v| v.to_owned())
            .collect::<Vec<_>>()
    }

    fn get_diff_position(&self, current_version_parts: &[Part]) -> Option<usize> {
        Version::from(&self.latest_installed_version).and_then(|newest_current_version| {
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

    fn build_version<R>(&self, version_parts: &[Part], version_range: R, delimiter_range: R) -> Result<String, Error>
    where
        R: std::ops::RangeBounds<usize>,
    {
        match (version_range.start_bound(), delimiter_range.start_bound()) {
            (Included(&v), Included(&d)) | (Excluded(&v), Excluded(&d)) if v != d => {
                panic!("{}", Error::VersionRangeStart)
            },
            _ => (),
        };
        match (version_range.end_bound(), delimiter_range.end_bound()) {
            (Included(&v), Included(&d)) | (Excluded(&v), Excluded(&d)) if v <= d => {
                panic!("{}", Error::VersionRangeEnd)
            },
            _ => (),
        };

        let version_parts = version_parts
            .range(&version_range)
            .ok_or(Error::IndexOutOfRange)?
            .map(|v| v.to_string());
        let delimiters = self
            .delimiters
            .range(&delimiter_range)
            .ok_or(Error::IndexOutOfRange)?
            .map(|v| v.to_string());

        Ok(version_parts.interleave(delimiters).join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn before(
        installed_versions: impl IntoIterator<Item = impl AsRef<str>>,
        current_version: &str,
    ) -> (VersionComparison, Vec<Part>) {
        let version = VersionComparison::new(installed_versions, current_version);
        let parts = Version::from(current_version).unwrap();
        let parts = parts.parts();

        (version, parts.to_owned())
    }

    #[test]
    fn generate_delimiters_should_return_delimiters_list() {
        assert_eq!(
            VersionComparison::get_delimiters("1.2_3-4"),
            [".".to_owned(), "_".to_owned(), "-".to_owned()]
        );
    }

    #[test]
    fn find_different_part_position_should_return_position() {
        let (version, ref parts) = before(vec!["2.0"], "1.0");
        assert_eq!(version.get_diff_position(parts), Some(0));

        let (version, ref parts) = before(vec!["1.0a"], "1.0b");
        assert_eq!(version.get_diff_position(parts), Some(2));

        let (version, ref parts) = before(vec!["1.0"], "1.0");
        assert_eq!(version.get_diff_position(parts), None);
    }

    #[test]
    fn build_version_should_return_string() {
        let (version, ref parts) = before(vec![""], "1.0.0_1");

        assert_eq!(version.build_version(parts, ..1, ..0).ok(), Some("1".to_owned()));
        assert_eq!(version.build_version(parts, ..2, ..1).ok(), Some("1.0".to_owned()));
        assert_eq!(version.build_version(parts, ..3, ..2).ok(), Some("1.0.0".to_owned()));
        assert_eq!(version.build_version(parts, ..4, ..3).ok(), Some("1.0.0_1".to_owned()));

        assert_eq!(version.build_version(parts, 0.., 0..).ok(), Some("1.0.0_1".to_owned()));
        assert_eq!(version.build_version(parts, 1.., 1..).ok(), Some("0.0_1".to_owned()));
        assert_eq!(version.build_version(parts, 2.., 2..).ok(), Some("0_1".to_owned()));
        assert_eq!(version.build_version(parts, 3.., 3..).ok(), Some("1".to_owned()));
    }

    #[test]
    #[should_panic(expected = "same")]
    fn build_version_should_panic_when_passed_two_ranges_start_is_not_same() {
        let (version, ref parts) = before(vec![""], "1.0.0_1");

        let _ = version.build_version(parts, 0.., 1..);
    }

    #[test]
    #[should_panic(expected = "greater than")]
    fn test_should_panic_build_version_first_range_end_not_greater_than_second_range_end() {
        let (version, ref parts) = before(vec![""], "1.0.0_1");

        let _ = version.build_version(parts, ..1, ..1);
    }

    #[test]
    fn colorize_should_return_version_colored() {
        assert_eq!(
            VersionComparison::new(&["1.0.0"], "2.0.0").colorize(),
            format!("{}{}", "", "2.0.0".color(VERSION_COLOR.major))
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0"], "1.1.0").colorize(),
            format!("{}{}", "1.", "1.0".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0"], "1.0.1").colorize(),
            format!("{}{}", "1.0.", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0_0"], "1.0.0_1").colorize(),
            format!("{}{}", "1.0.0_", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0-0"], "1.0.0-1").colorize(),
            format!("{}{}", "1.0.0-", "1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new(&["2.4+20150115"], "2.4+20151223_1").colorize(),
            format!("{}{}", "2.4+", "20151223_1".color(VERSION_COLOR.other))
        );
        assert_eq!(
            VersionComparison::new(&["3.1"], "3.2a").colorize(),
            format!("{}{}", "3.", "2a".color(VERSION_COLOR.minor))
        );
        assert_eq!(
            VersionComparison::new(&["r2917_1"], "r2999").colorize(),
            format!("{}{}", "", "r2999".color(VERSION_COLOR.major))
        );
        assert_eq!(
            VersionComparison::new(&["3.4.1,3041"], "3.4.2,3043").colorize(),
            format!("{}{}", "3.4.", "2,3043".color(VERSION_COLOR.other))
        );
    }
}
