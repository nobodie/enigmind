use serde::{Deserialize, Serialize};

use crate::{column::Column, error::EnigmindError, setup::GameConfiguration};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Code(pub Vec<u8>);

impl From<u32> for Code {
    fn from(code: u32) -> Self {
        let mut c = code;
        let mut v = Vec::new();

        while c > 0 {
            v.push((c % 10) as u8);

            c /= 10;
        }

        v.reverse();
        Code::new(v)
    }
}

impl Code {
    pub fn new(v: Vec<u8>) -> Self {
        Self(v)
    }

    pub fn get(&self, c: Column) -> Result<u8, EnigmindError> {
        let index: usize = c.into();
        self.0
            .get(index)
            .ok_or(EnigmindError::ColumnIndexOutOfBounds)
            .cloned()
    }

    pub fn get_shift(&self, gc: &GameConfiguration) -> u32 {
        let mut shift = 0;
        let mut current_column = 0;

        self.0.iter().rev().for_each(|x| {
            shift += (*x as u32) * (gc.base as u32).pow(current_column);
            current_column += 1;
        });

        shift
    }

    pub fn from_shift(shift: u32, gc: &GameConfiguration) -> Self {
        let mut code_vec = Vec::new();
        for column in 0..gc.column_count {
            code_vec.push(((shift / ((gc.base as u32).pow(column as u32))) % gc.base as u32) as u8);
        }
        code_vec.reverse();
        Code::new(code_vec)
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.0 {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}
