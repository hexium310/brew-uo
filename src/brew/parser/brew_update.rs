use crate::error::Error;
use itertools::Itertools;
use regex::Regex;

trait BrewUpdateParser {
    fn messages(update_result_text: &str) -> Result<Vec<String>, Error>;
    fn information(update_result_text: &str) -> Vec<(Vec<&str>, Vec<&str>)>;
}

#[derive(Clone, Debug)]
pub struct BrewUpdateData<'a> {
    messages: Vec<String>,
    information: Vec<(Vec<&'a str>, Vec<&'a str>)>,
}

impl<'a> BrewUpdateData<'a> {
    pub fn new(update_result_text: &str) -> BrewUpdateData {
        let messages = BrewUpdateData::messages(update_result_text).unwrap();
        let information = BrewUpdateData::information(update_result_text);

        BrewUpdateData { messages, information }
    }
}

impl<'a> BrewUpdateParser for BrewUpdateData<'a> {
    fn messages(update_result_text: &str) -> Result<Vec<String>, Error> {
        Ok(
            Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)")?
                .captures_iter(update_result_text)
                .map(|captures| (&captures[0]).to_owned())
                .collect::<Vec<_>>(),
        )
    }

    fn information(update_result_text: &str) -> Vec<(Vec<&str>, Vec<&str>)> {
        update_result_text
            .lines()
            // When the line starts with "==>", returns (true, value).
            .group_by(|v| v.find("==>").is_some())
            .into_iter()
            .map(|(k, v)| (k, v.collect::<Vec<_>>()))
            .batching(|it| {
                while let Some((kind_bool, kind_value)) = it.next() {
                    if !kind_bool {
                        continue;
                    } else {
                        return match it.next() {
                            Some((_, formulae_value)) => Some((kind_value, formulae_value)),
                            None => None,
                        }
                    }
                }

                None
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages() {
        let text = indoc!(
            r#"
                Updated 1 tap (homebrew/core).
                Already up-to-date.
                No changes to formulae.
                ==> Updated Formulae
                rust
                typescript
            "#
        );
        let messages = BrewUpdateData::messages(text);

        assert_eq!(
            messages.ok(),
            Some(vec![
                "Updated 1 tap (homebrew/core).".to_owned(),
                "Already up-to-date.".to_owned(),
                "No changes to formulae.".to_owned(),
            ])
        );
        assert_eq!(BrewUpdateData::messages("").ok(), Some(vec![]));
    }

    #[test]
    fn update_information() {
        let text = indoc!(
            r#"
                Updated 1 tap (homebrew/core).
                Already up-to-date.
                No changes to formulae.
                ==> Updated Formulae
                php
                rust
                typescript
                vim
                ==> Deleted Formulae
                go
                python
                ruby
                ==> TEST
                test1
                test2
            "#
        );
        let information = BrewUpdateData::information(text);
        let mut iter = information.iter();

        assert_eq!(
            iter.next(),
            Some(&(vec!["==> Updated Formulae"], vec!["php", "rust", "typescript", "vim"]))
        );
        assert_eq!(
            iter.next(),
            Some(&(vec!["==> Deleted Formulae"], vec!["go", "python", "ruby"]))
        );
        assert_eq!(iter.next(), Some(&(vec!["==> TEST"], vec!["test1", "test2"])));
        assert_eq!(iter.next(), None);
    }
}
