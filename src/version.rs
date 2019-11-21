use crate::error::Error;
use colored::{Color, Colorize};
use itertools::EitherOrBoth::Both;
use itertools::Itertools;
use version_compare::{CompOp, VersionCompare, VersionPart};

pub trait VersionTrait {
    fn parse(&self) -> Result<String, Error>;
    fn find_different_part_position(&self, latest_version_parts: &[VersionPart]) -> Option<usize>;
    fn build_version(&self, version_parts: &[VersionPart], delimiters: &[String]) -> String;
}

#[derive(Clone, Debug)]
pub struct Version {
    current_versions: Vec<String>,
    latest_version: String,
    delimiters: Vec<String>,
}

impl Version {
    pub fn new(current_versions: &[&str], latest_version: &str) -> Version {
        let delimiters = Version::delimiters(latest_version);

        Version {
            current_versions: current_versions.iter().map(|&v| v.to_owned()).collect::<Vec<_>>(),
            latest_version: latest_version.to_owned(),
            delimiters,
        }
    }

    pub fn delimiters(version_str: &str) -> Vec<String> {
        let delimiter_chars = ['.', '_'];

        version_str
            .matches(|version_char| {
                delimiter_chars
                    .iter()
                    .any(|&delimiter_char| delimiter_char == version_char)
            })
            .map(|v| v.to_owned())
            .collect::<Vec<_>>()
    }
}

impl VersionTrait for Version {
    fn parse(&self) -> Result<String, Error> {
        version_compare::Version::from(&self.latest_version).map_or_else(
            || Ok(self.latest_version.clone()),
            |latest_version| {
                let latest_version_parts = latest_version.parts();
                let different_part_position = self.find_different_part_position(latest_version_parts);

                Ok(different_part_position.map_or_else(
                    || latest_version.to_string(),
                    |position| {
                        let (latest_version_parts_without_change, between_delimiter) =
                            if let Some(p) = position.checked_sub(1) {
                                (
                                    self.build_version(&latest_version_parts[..position], &self.delimiters[..p]),
                                    self.delimiters[p].to_owned(),
                                )
                            } else {
                                ("".to_owned(), "".to_owned())
                            };
                        let latest_version_parts_with_change = self
                            .build_version(&latest_version_parts[position..], &self.delimiters[position..])
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
                ))
            },
        )
    }

    fn find_different_part_position(&self, latest_version_parts: &[VersionPart]) -> Option<usize> {
        version_compare::Version::from(self.current_versions.last().unwrap()).map_or_else(
            || Some(1),
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

    fn build_version(&self, version_parts: &[VersionPart], delimiters: &[String]) -> String {
        version_parts
            .iter()
            .map(|v| v.to_string())
            .interleave(delimiters.iter().map(|v| v.to_owned()))
            .join("")
    }
}
