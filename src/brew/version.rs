use crate::error::Error;
use crate::range::*;
use std::ops::Bound::*;
use colored::{Color, Colorize};
use itertools::EitherOrBoth::Both;
use itertools::Itertools;
use version_compare::{Cmp, Part};

#[derive(Clone, Debug)]
pub struct VersionComparison {
    latest_installed_version: String,
    current_version: String,
    delimiters: Vec<String>,
}

impl VersionComparison {
    pub fn new(installed_versions: impl IntoIterator<Item = impl AsRef<str>>, current_version: &str) -> VersionComparison {
        let delimiters = VersionComparison::get_delimiters(current_version);
        let latest_installed_version = VersionComparison::get_latest_installed_version(installed_versions);

        VersionComparison {
            latest_installed_version,
            current_version: current_version.to_owned(),
            delimiters,
        }
    }

    pub fn colorize(&self) -> String {
        version_compare::Version::from(&self.current_version).map_or_else(
            || self.current_version.red().to_string(),
            |current_version| {
                let current_version_parts = current_version.parts();
                let different_part_position = self.find_different_part_position(current_version_parts);

                different_part_position.map_or_else(
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
                                0 => Color::Red,
                                1 => Color::Blue,
                                _ => Color::Green,
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

    fn get_latest_installed_version(latest_versions: impl IntoIterator<Item = impl AsRef<str>>) -> String {
        let latest_versions = latest_versions.into_iter().map(|v| v.as_ref().to_owned()).collect::<Vec<String>>();
        let latest_version = latest_versions.last().unwrap();

        latest_version.to_string()
    }

    fn get_delimiters(version_str: &str) -> Vec<String> {
        let delimiter_chars = ['.', '_', '-', '+'];

        version_str
            .matches(|version_char| {
                delimiter_chars
                    .iter()
                    .any(|&delimiter_char| delimiter_char == version_char)
            })
            .map(|v| v.to_owned())
            .collect::<Vec<_>>()
    }

    fn find_different_part_position(&self, current_version_parts: &[Part]) -> Option<usize> {
        version_compare::Version::from(&self.latest_installed_version).and_then(
            |newest_current_version| {
                current_version_parts
                    .iter()
                    .zip_longest(newest_current_version.parts().iter())
                    .position(|v| match v {
                        Both(left, right) => {
                            version_compare::compare_to(&left.to_string(), &right.to_string(), Cmp::Ne)
                                .unwrap_or_else(|_| left.to_string() != right.to_string())
                        },
                        _ => true,
                    })
            },
        )
    }

    fn build_version<R>(
        &self,
        version_parts: &[Part],
        version_range: R,
        delimiter_range: R,
    ) -> Result<String, Error>
    where
        R: std::ops::RangeBounds<usize>,
    {
        match (version_range.start_bound(), delimiter_range.start_bound()) {
            (Included(&v), Included(&d)) | (Excluded(&v), Excluded(&d)) if v != d => panic!("{}", Error::VersionRangeStartError),
            _ => (),
        };

        match (version_range.end_bound(), delimiter_range.end_bound()) {
            (Included(&v), Included(&d)) | (Excluded(&v), Excluded(&d)) if v <= d => panic!("{}", Error::VersionRangeEndError),
            _ => (),
        };

        let version_parts_with_range = version_parts
            .range(&version_range)
            .ok_or(Error::IndexOutOfRange)?
            .map(|v| v.to_string());
        let delimiters_with_range = self
            .delimiters
            .range(&delimiter_range)
            .ok_or(Error::IndexOutOfRange)?
            .map(|v| v.to_string());

        Ok(version_parts_with_range.interleave(delimiters_with_range).join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn before<'a>() -> (VersionComparison, Vec<Part<'a>>) {
        let current_version = "1.0.0_1";
        let version = VersionComparison::new(&[""], current_version);
        let mut parts: Vec<Part> = vec![];
        for part in version_compare::Version::from(current_version).unwrap().parts() {
            parts.push(match part {
                Part::Number(v) => Part::Number(*v),
                Part::Text(v) => Part::Text(v),
            })
        }

        (version, parts)
    }

    #[test]
    fn generate_delimiters_should_return_delimiters_list() {
        assert_eq!(VersionComparison::get_delimiters("1.2_3-4"), [".".to_owned(), "_".to_owned(), "-".to_owned()]);
    }

    #[test]
    fn find_different_part_position_should_return_position() {
        let current_version = "1.0";
        let version = VersionComparison::new(&["2.0"], current_version);
        let v = version_compare::Version::from(current_version).unwrap();
        let current_version_parts = v.parts();

        assert_eq!(version.find_different_part_position(current_version_parts), Some(0));

        let current_version = "1.0b";
        let version = VersionComparison::new(&["1.0a"], current_version);
        let v = version_compare::Version::from(current_version).unwrap();
        let current_version_parts = v.parts();

        assert_eq!(version.find_different_part_position(current_version_parts), Some(2));

        let current_version = "1.0";
        let version = VersionComparison::new(&["1.0"], current_version);
        let v = version_compare::Version::from(current_version).unwrap();
        let current_version_parts = v.parts();

        assert_eq!(version.find_different_part_position(current_version_parts), None);
    }

    #[test]
    fn build_version_should_return_string() {
        let before = before();
        let version = before.0;
        let parts = &before.1;

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
        let before = before();
        let version = before.0;
        let parts = &before.1;

        let _ = version.build_version(parts, 0.., 1..);
    }

    #[test]
    #[should_panic(expected = "greater than")]
    fn test_should_panic_build_version_first_range_end_not_greater_than_second_range_end() {
        let before = before();
        let version = before.0;
        let parts = &before.1;

        let _ = version.build_version(parts, ..1, ..1);
    }

    #[test]
    fn colorize_should_return_version_colored() {
        assert_eq!(
            VersionComparison::new(&["1.0.0"], "2.0.0").colorize(),
            format!("{}{}", "", "2.0.0".red())
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0"], "1.1.0").colorize(),
            format!("{}{}", "1.", "1.0".blue())
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0"], "1.0.1").colorize(),
            format!("{}{}", "1.0.", "1".green())
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0_0"], "1.0.0_1").colorize(),
            format!("{}{}", "1.0.0_", "1".green())
        );
        assert_eq!(
            VersionComparison::new(&["1.0.0-0"], "1.0.0-1").colorize(),
            format!("{}{}", "1.0.0-", "1".green())
        );
        assert_eq!(
            VersionComparison::new(&["2.4+20150115"], "2.4+20151223_1").colorize(),
            format!("{}{}", "2.4+", "20151223_1".green())
        );
        assert_eq!(
            VersionComparison::new(&["3.1"], "3.2a").colorize(),
            format!("{}{}", "3.", "2a".blue())
        );
        assert_eq!(
            VersionComparison::new(&["r2917_1"], "r2999").colorize(),
            format!("{}{}", "", "r2999".red())
        );
    }
}
