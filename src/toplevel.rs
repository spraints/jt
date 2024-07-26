use std::path::{Path, PathBuf};

use crate::errs;

// This exists in order to break a dependency cycle. It might be better to create instead a utility
// object that could be inside of a Journal and be passed to Book, etc.
pub trait JournalTopLevel: Clone {
    fn path(&self) -> PathBuf;
    fn commit_file(&self, path: impl AsRef<Path>) -> errs::SimpleResult;
}

pub trait JournalEntity {
    fn path(&self) -> PathBuf;
}
