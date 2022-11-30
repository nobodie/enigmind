use serde::{Deserialize, Serialize};

use crate::column::Column;
use std::{
    collections::HashSet,
    fmt,
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSet(HashSet<Column>);

impl ColumnSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ColumnSet {
    type Target = HashSet<Column>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ColumnSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashSet<Column>> for ColumnSet {
    fn from(value: HashSet<Column>) -> Self {
        Self(value)
    }
}

impl From<ColumnSet> for HashSet<Column> {
    fn from(value: ColumnSet) -> Self {
        value.0
    }
}

impl PartialEq for ColumnSet {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for ColumnSet {}

impl Hash for ColumnSet {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
        self.0.hasher();
    }
}

impl fmt::Display for ColumnSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for r in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", r)?;
            first = false;
        }
        write!(f, "]")?;

        Ok(())
    }
}
