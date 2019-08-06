pub mod repository;
pub mod store;
mod timesheet;

pub use crate::repository::Repository;
pub use crate::store::{
    meta::Meta,
    patch::{Patch, PatchRef},
    Store,
};
pub use crate::timesheet::{Event, Timesheet};

pub type EventRef = String;
pub type Tag = String;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
