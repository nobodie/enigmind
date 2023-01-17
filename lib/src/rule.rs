use crate::{
    code::Code, columns::ColumnSet, error::EnigmindError, rules::Rules, setup::GameConfiguration,
};
use nbitmask::BitMask;
use serde::{Deserialize, Serialize};
use std::{fmt, vec};

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Operator {
    Pair,
    Impair,
    Lowest,
    Highest,
    SumBelow(u8),
    SumEquals(u8),
    SumAbove(u8),
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Pair => write!(f, "even"),
            Operator::Impair => write!(f, "odd"),
            Operator::Lowest => write!(f, "lowest"),
            Operator::Highest => write!(f, "highest"),
            Operator::SumBelow(_) => write!(f, "below"),
            Operator::SumEquals(_) => write!(f, "equal to"),
            Operator::SumAbove(_) => write!(f, "above"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]

pub enum Rule {
    MatchesOp(Operator, ColumnSet),
    XColumnsEquals(u8, u8),
}

impl Rule {
    pub fn evaluate(&self, code: Code) -> Result<bool, EnigmindError> {
        Ok(match self {
            Rule::XColumnsEquals(count, value) => {
                code.0.iter().filter(|x| **x == *value).count() == (*count as usize)
            }
            Rule::MatchesOp(op, columns) => match op {
                Operator::Highest => {
                    let mut res = true;
                    for col in columns.iter() {
                        let col_value = code.get(*col)?;
                        let max_value = *(code.0.iter().max().unwrap());
                        let cnt = code.0.iter().filter(|x| **x == max_value).count();

                        if col_value != max_value || cnt != 1 {
                            res = false;
                        }
                    }
                    res
                }
                Operator::Lowest => {
                    let mut res = true;
                    for col in columns.iter() {
                        let col_value = code.get(*col)?;
                        let min_value = *(code.0.iter().min().unwrap());
                        let cnt = code.0.iter().filter(|x| **x == min_value).count();

                        if col_value != min_value || cnt != 1 {
                            res = false;
                        }
                    }
                    res
                }
                Operator::Pair => {
                    let mut res = true;
                    for col in columns.iter() {
                        res &= (code.get(*col)? % 2) == 0;
                    }
                    res
                }
                Operator::Impair => {
                    let mut res = true;
                    for col in columns.iter() {
                        res &= (code.get(*col)? % 2) == 1;
                    }
                    res
                }
                Operator::SumBelow(value) => {
                    let mut sum = 0;
                    for col in columns.iter() {
                        sum += code.get(*col)?;
                    }
                    sum < *value
                }

                Operator::SumEquals(value) => {
                    let mut sum = 0;
                    for col in columns.iter() {
                        sum += code.get(*col)?;
                    }
                    sum == *value
                }
                Operator::SumAbove(value) => {
                    let mut sum = 0;
                    for col in columns.iter() {
                        sum += code.get(*col)?;
                    }
                    sum > *value
                }
            },
        })
    }

    pub fn get_mask(&self, config: &GameConfiguration) -> Result<BitMask<u64>, EnigmindError> {
        let n = config.solution_count() as usize;
        let mut mask = BitMask::zeros(n);

        for i in 0..n {
            let code = Code::from_shift(i as u32, config);
            mask.set(i, self.evaluate(code)?)?;
        }

        Ok(mask)
    }

    pub fn get_similar(&self, gc: &GameConfiguration) -> Vec<(String, Rules)> {
        let mut v = Vec::new();

        match &self {
            Rule::MatchesOp(op, columns) => match op {
                Operator::Pair | Operator::Impair => {
                    v.push((
                        "Column is pair or impair".to_string(),
                        vec![
                            Rule::MatchesOp(Operator::Pair, columns.clone()),
                            Rule::MatchesOp(Operator::Impair, columns.clone()),
                        ]
                        .into(),
                    ));

                    v.push((
                        "One of the column is pair".to_string(),
                        gc.get_column_combinations(columns.len() as u8)
                            .iter()
                            .map(|c| Rule::MatchesOp(*op, c.clone()))
                            .collect(),
                    ));
                }
                Operator::Lowest => v.push((
                    "One of the column is the lowest".to_string(),
                    gc.get_column_combinations(columns.len() as u8)
                        .iter()
                        .map(|c| Rule::MatchesOp(*op, c.clone()))
                        .collect(),
                )),
                Operator::Highest => v.push((
                    "One of the column is the highest".to_string(),
                    gc.get_column_combinations(columns.len() as u8)
                        .iter()
                        .map(|c| Rule::MatchesOp(*op, c.clone()))
                        .collect(),
                )),
                Operator::SumBelow(value)
                | Operator::SumEquals(value)
                | Operator::SumAbove(value) => {
                    v.push((
                        format!(
                            "Column(s) {} sum is below, equal or above {}",
                            columns, *value
                        ),
                        vec![
                            Rule::MatchesOp(Operator::SumBelow(*value), columns.clone()),
                            Rule::MatchesOp(Operator::SumEquals(*value), columns.clone()),
                            Rule::MatchesOp(Operator::SumAbove(*value), columns.clone()),
                        ]
                        .into(),
                    ));

                    v.push((
                        format!("Sum of {} columns is {} {}", columns.len(), *op, *value),
                        gc.get_column_combinations(columns.len() as u8)
                            .iter()
                            .map(|cs| Rule::MatchesOp(*op, cs.clone()))
                            .collect(),
                    ));
                }
            },
            Rule::XColumnsEquals(_, value) => {
                let mut equal_rules = Vec::new();
                for i in 0..gc.column_count + 1 {
                    equal_rules.push(Rule::XColumnsEquals(i, *value));
                }

                v.push((
                    format!("There are X columns that equals {}", *value),
                    equal_rules.into(),
                ));
            }
        }
        v
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Rule::XColumnsEquals(count, value) => format!("XColumnsEquals({count}, {value})"),

            Rule::MatchesOp(op, columns) => match op {
                Operator::Lowest => format!("IsLowest({columns})"),
                Operator::Highest => format!("IsHighest({columns})"),
                Operator::Pair => format!("IsPair({columns})"),
                Operator::Impair => format!("IsImpair({columns})"),
                Operator::SumBelow(value) => format!("SumBelow({columns}, {value})"),
                Operator::SumEquals(value) => format!("SumEquals({columns}, {value})"),
                Operator::SumAbove(value) => format!("SumAbove({columns}, {value})"),
            },
        };

        write!(f, "{text}")
    }
}
