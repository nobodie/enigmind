use crate::{error::EnigmindError, rule::Rule, setup::GameConfiguration};
use nbitmask::BitMask;
use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Verifier {
    pub rule: Rule,
    pub mask: BitMask<u64>,
}

impl Verifier {
    pub fn new(gc: &GameConfiguration, rule: Rule) -> Result<Self, EnigmindError> {
        let mask = rule.get_mask(gc)?;
        Ok(Self { rule, mask })
    }
}

impl fmt::Display for Verifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", &self.rule, &self.mask)?;
        Ok(())
    }
}

pub struct Verificators(Vec<Verifier>);

impl From<Verificators> for Vec<Verifier> {
    fn from(vs: Verificators) -> Self {
        vs.0
    }
}

impl From<Vec<Verifier>> for Verificators {
    fn from(v: Vec<Verifier>) -> Self {
        Self(v)
    }
}

impl Deref for Verificators {
    type Target = Vec<Verifier>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Verificators {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in &self.0 {
            writeln!(f, "{}", v)?;
        }
        Ok(())
    }
}
