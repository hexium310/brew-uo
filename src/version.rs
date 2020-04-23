use crate::error::Error;
use crate::range::*;
use std::ops::Bound::*;
use colored::{Color, Colorize};
use itertools::EitherOrBoth::Both;
use itertools::Itertools;
use version_compare::{CompOp, VersionCompare, VersionPart};

#[derive(Clone, Debug)]
pub struct Version {
    current_versions: Vec<String>,
    latest_version: String,
    delimiters: Vec<String>,
}

impl Version {
    pub fn new<I, T>(current_versions: I, latest_version: &str) -> Version
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let delimiters = Self::generate_delimiters(latest_version);

        Version {
            current_versions: current_versions.into_iter().map(|v| v.as_ref().to_owned()).collect(),
            latest_version: latest_version.to_owned(),
            delimiters,
        }
    }

    pub fn colorize(&self) -> String {
        version_compare::Version::from(&self.latest_version).map_or_else(
            || self.latest_version.clone(),
            |latest_version| {
                let latest_version_parts = latest_version.parts();
                let different_part_position = self.find_different_part_position(latest_version_parts);

                different_part_position.map_or_else(
                    || latest_version.to_string(),
                    |position| {
                        let (latest_version_parts_without_change, between_delimiter) =
                            position.checked_sub(1).map_or_else(
                                || ("".to_owned(), "".to_owned()),
                                |delimiters_position| {
                                    (
                                        self.build_version(latest_version_parts, ..position, ..delimiters_position)
                                            .unwrap(),
                                        self.delimiters[delimiters_position].to_owned(),
                                    )
                                },
                            );
                        let latest_version_parts_with_change = self
                            .build_version(latest_version_parts, position.., position..)
                            .unwrap()
                            .color(match position {
                                0 => Color::Red,
                                1 => Color::Blue,
                                _ => Color::Green,
                            })
                            .to_string();

                        format!(
                            "{}{}{}",
                            latest_version_parts_without_change, between_delimiter, latest_version_parts_with_change,
                        )
                    },
                )
            },
        )
    }

    fn generate_delimiters(version_str: &str) -> Vec<String> {
        let delimiter_chars = ['.', '_', '-'];

        version_str
            .matches(|version_char| {
                delimiter_chars
                    .iter()
                    .any(|&delimiter_char| delimiter_char == version_char)
            })
            .map(|v| v.to_owned())
            .collect::<Vec<_>>()
    }

    fn find_different_part_position(&self, latest_version_parts: &[VersionPart]) -> Option<usize> {
        version_compare::Version::from(self.current_versions.last().unwrap_or(&"".to_owned())).and_then(
            |newest_current_version| {
                latest_version_parts
                    .iter()
                    .zip_longest(newest_current_version.parts().iter())
                    .position(|v| match v {
                        Both(left, right) => {
                            VersionCompare::compare_to(&left.to_string(), &right.to_string(), &CompOp::Ne).unwrap()
                        },
                        _ => true,
                    })
            },
        )
    }

    fn build_version<R>(
        &self,
        version_parts: &[VersionPart],
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

    fn before<'a>() -> (Version, Vec<version_compare::version_part::VersionPart<'a>>) {
        let latest_version = "1.0.0_1";
        let version = Version::new(&[""], latest_version);
        let mut parts: Vec<VersionPart> = vec![];
        for part in version_compare::Version::from(latest_version).unwrap().parts() {
            parts.push(match part {
                VersionPart::Number(v) => VersionPart::Number(*v),
                VersionPart::Text(v) => VersionPart::Text(v),
            })
        }

        (version, parts)
    }

    #[test]
    fn generate_delimiters_should_return_delimiters_list() {
        assert_eq!(Version::generate_delimiters("1.2_3-4"), [".".to_owned(), "_".to_owned(), "-".to_owned()]);
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
}
