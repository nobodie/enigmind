use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::criteria::Criteria;

#[derive(Clone, Serialize, Deserialize)]

pub struct Criterias(Vec<Criteria>);

impl From<Criterias> for Vec<Criteria> {
    fn from(rs: Criterias) -> Self {
        rs.0
    }
}

impl From<Vec<Criteria>> for Criterias {
    fn from(v: Vec<Criteria>) -> Self {
        Self(v)
    }
}

impl FromIterator<Criteria> for Criterias {
    fn from_iter<T: IntoIterator<Item = Criteria>>(iter: T) -> Self {
        let mut v = Vec::new();

        for i in iter {
            v.push(i);
        }
        Self(v)
    }
}

impl Deref for Criterias {
    type Target = Vec<Criteria>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Criterias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.0 {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}
