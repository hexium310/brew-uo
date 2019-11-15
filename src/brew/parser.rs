pub mod brew_outdated;
pub mod brew_update;

pub use brew_outdated::*;
pub use brew_update::*;

pub(crate) trait Parser {
    type IteratorItem;
    type Items: Iterator<Item = Self::IteratorItem>;

    fn items(&self) -> Self::Items;
}
