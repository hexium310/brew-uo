pub mod brew_outdated;

pub use brew_outdated::*;

pub(crate) trait Parser {
    type IteratorItem;
    type Items: Iterator<Item = Self::IteratorItem>;

    fn items(&self) -> Self::Items;
}
