use std::{fmt, hash::Hash};

use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct Column(u8);

impl From<Column> for u8 {
    fn from(c: Column) -> Self {
        c.0
    }
}

impl From<u8> for Column {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl From<Column> for usize {
    fn from(c: Column) -> Self {
        c.0 as usize
    }
}

impl Column {}

/*impl Hash for Column {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}*/

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (self.0 + 65) as char)
    }
}

/*#[derive(PartialEq, Clone)]
pub struct Columns(Vec<Column>);

impl From<Columns> for Vec<Column> {
    fn from(rs: Columns) -> Self {
        rs.0
    }
}

impl From<Vec<Column>> for Columns {
    fn from(v: Vec<Column>) -> Self {
        Self(v)
    }
}

impl Deref for Columns {
    type Target = Vec<Column>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Columns {
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
*/
