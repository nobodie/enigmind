use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{rules::Rules, verifier::Verifier};

#[derive(Clone, Serialize, Deserialize)]
pub struct Criteria {
    pub verif: Verifier,
    pub description: String,
    pub rules: Rules,
}

impl fmt::Display for Criteria {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Criteria : {}.", self.description)?;
        writeln!(f, "Rules : {} {}.", self.verif.rule, self.verif.mask)?;
        for rule in self.rules.iter() {
            write!(f, "\t{rule}")?;
            if *rule == self.verif.rule {
                write!(f, " (*)")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
