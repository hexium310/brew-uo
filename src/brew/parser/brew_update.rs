use super::Parser;
use crate::error::Error;
use itertools::Itertools;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct BrewUpdateData {
    text: String,
}

impl BrewUpdateData {
    pub(crate) fn new(text: &str) -> Self {
        BrewUpdateData { text: text.to_owned() }
    }

    pub(crate) fn messages(&self) -> Result<Vec<String>, Error> {
        Ok(
            Regex::new(r"(?m)^(?:Updated .+|Already up-to-date\.|No changes to formulae\.)$(?-m)")
                .unwrap()
                .captures_iter(&self.text)
                .map(|captures| (&captures[0]).to_owned())
                .collect::<Vec<_>>(),
        )
    }
}

impl Parser for BrewUpdateData {
    type IteratorItem = (Vec<String>, Vec<String>);
    type Items = impl Iterator<Item = Self::IteratorItem>;

    fn items(&self) -> Self::Items {
        self.text
            .lines()
            // When the line starts with "==>", returns (true, value).
            .group_by(|v| v.find("==>").is_some())
            .into_iter()
            .map(|(k, v)| (k, v.map(String::from).collect::<Vec<_>>()))
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
            .into_iter()
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
        let messages = BrewUpdateData::new(text).messages();

        assert_eq!(
            messages.ok(),
            Some(vec![
                "Updated 1 tap (homebrew/core).".to_owned(),
                "Already up-to-date.".to_owned(),
                "No changes to formulae.".to_owned(),
            ])
        );
        assert_eq!(BrewUpdateData::new("").messages().ok(), Some(vec![]));
    }

    #[test]
    fn update_items() {
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
        let mut items = BrewUpdateData::new(text).items();

        assert_eq!(
            items.next(),
            Some((
                vec!["==> Updated Formulae".to_owned()],
                vec![
                    "php".to_owned(),
                    "rust".to_owned(),
                    "typescript".to_owned(),
                    "vim".to_owned()
                ]
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                vec!["==> Deleted Formulae".to_owned()],
                vec!["go".to_owned(), "python".to_owned(), "ruby".to_owned()]
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                vec!["==> TEST".to_owned()],
                vec!["test1".to_owned(), "test2".to_owned()]
            ))
        );
        assert_eq!(items.next(), None);

        let mut empty = BrewUpdateData::new("").items();

        assert_eq!(empty.next(), None);
    }
}
