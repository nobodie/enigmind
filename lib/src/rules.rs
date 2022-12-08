use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::rule::Rule;

#[derive(Clone, Serialize, Deserialize)]
pub struct Rules(Vec<Rule>);

impl From<Rules> for Vec<Rule> {
    fn from(rs: Rules) -> Self {
        rs.0
    }
}

impl From<Vec<Rule>> for Rules {
    fn from(v: Vec<Rule>) -> Self {
        Self(v)
    }
}

impl FromIterator<Rule> for Rules {
    fn from_iter<T: IntoIterator<Item = Rule>>(iter: T) -> Self {
        let mut v = Vec::new();

        for i in iter {
            v.push(i);
        }
        Self(v)
    }
}

impl Deref for Rules {
    type Target = Vec<Rule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Rules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in &self.0 {
            writeln!(f, "{r}")?;
        }
        Ok(())
    }
}
