use crate::{
    code::Code,
    column::Column,
    columns::ColumnSet,
    criteria::Criteria,
    criterias::Criterias,
    error::EnigmindError,
    rule::{Operator, Rule},
    rules::Rules,
    term_format::TermFormat,
    verifier::{Verificators, Verifier},
};
use itertools::Itertools;
use nbitmask::BitMask;
use pad::PadStr;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt, ops::Deref};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfiguration {
    pub column_count: u8,
    pub base: u8,
    pub min_difficulty: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub configuration: GameConfiguration,
    pub criterias: Criterias,
    pub code: Code,
}

impl Game {
    pub fn is_solution_compatible(&self, code: &Code) -> bool {
        if code.0.len() != self.configuration.column_count as usize {
            return false;
        }

        if code.0.iter().any(|&f| f >= self.configuration.base) {
            return false;
        }
        true
    }

    pub fn to_column_index(&self, column: char) -> u8 {
        return (column as u8) - 65;
    }

    pub fn is_column_compatible(&self, column: char) -> bool {
        if (column as u8) < 65 {
            return false;
        }
        return (column as u8) - 65 < self.configuration.column_count;
    }

    pub fn is_value_compatible(&self, value: u8) -> bool {
        return value < self.configuration.base;
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Game : {}", self.configuration)?;
        write!(f, "{}", self.criterias)?;
        write!(f, "Code to find : {}", self.code)
    }
}

impl GameConfiguration {
    pub fn solution_count(&self) -> u32 {
        (self.base as u32).pow(self.column_count as u32)
    }

    pub fn get_all_columns(&self) -> Vec<Column> {
        let mut v = Vec::new();
        for i in 0..self.column_count {
            v.push(Column::from(i));
        }
        v
    }

    pub fn get_all_column_pairs(&self) -> Vec<(Column, Column)> {
        let mut v = Vec::new();
        for i in 0..self.column_count {
            for j in i + 1..self.column_count {
                v.push((Column::from(i), Column::from(j)));
            }
        }
        v
    }

    pub fn get_all_column_combinations(&self) -> HashSet<ColumnSet> {
        let mut all_cartesian_prods = HashSet::new();

        let mut multi_prod = (0..self.column_count)
            .map(|_| 0..self.column_count)
            .multi_cartesian_product();
        let mut opt = multi_prod.next();
        while let Some(p) = opt {
            let hc: ColumnSet = HashSet::from_iter(p.iter().map(|i| Column::from(*i))).into();

            all_cartesian_prods.insert(hc);

            opt = multi_prod.next();
        }
        all_cartesian_prods
    }

    pub fn get_column_combinations(&self, length: u8) -> HashSet<ColumnSet> {
        let mut res = self.get_all_column_combinations();

        res.retain(|cs| cs.len() == length as usize);
        res
    }

    /*pub fn get_all_column_combinations(&self) -> HashSet<ColumnSet> {
        (1..=self.column_count)
            .flat_map(|i| self.get_column_combinations(i))
            .collect()
    }

    pub fn get_column_combinations(&self, length: u8) -> HashSet<ColumnSet> {
        fn combinations_rec(gc: &GameConfiguration, length: u8, l: u8) -> HashSet<ColumnSet> {
            let mut res = HashSet::new();

            if l == length {
                res.extend((0..gc.column_count).map(|i| {
                    let mut h = HashSet::new();
                    h.insert(i.into());
                    h.into()
                }));
            } else {
                for i in l + 1..=gc.column_count {
                    res.extend(
                        combinations_rec(&gc, length, l + 1)
                            .into_iter()
                            .map(|mut cs| {
                                cs.insert(i.into());
                                cs
                            }),
                    );
                }
            }
            dbg!(l);
            println!("{:?}", res.clone());
            res
        }

        combinations_rec(self, length, 0)
    }*/
}

impl fmt::Display for GameConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{} columns between 0 and {} ({} possibilities)}}",
            self.column_count,
            self.base,
            self.solution_count()
        )
    }
}

fn generate_game_configuration(
    base: u8,
    column_count: u8,
    difficulty_pct: u8,
) -> GameConfiguration {
    GameConfiguration {
        column_count,
        base,
        min_difficulty: difficulty_pct.clamp(0, 100),
    }
}

fn generate_rules(gc: &GameConfiguration) -> Result<Rules, EnigmindError> {
    let mut rules = Vec::new();

    gc.get_column_combinations(1).iter().for_each(|cs| {
        rules.push(Rule::MatchesOp(Operator::Pair, cs.clone()));
        rules.push(Rule::MatchesOp(Operator::Impair, cs.clone()));
        rules.push(Rule::MatchesOp(Operator::Lowest, cs.clone()));
        rules.push(Rule::MatchesOp(Operator::Highest, cs.clone()));
    });

    for c_cart_prod in gc.get_all_column_combinations() {
        for base in 0..((c_cart_prod.clone().len() as u8) * gc.base) {
            rules.push(Rule::MatchesOp(
                Operator::SumBelow(base),
                c_cart_prod.clone(),
            ));
            rules.push(Rule::MatchesOp(
                Operator::SumEquals(base),
                c_cart_prod.clone(),
            ));
            rules.push(Rule::MatchesOp(
                Operator::SumAbove(base),
                c_cart_prod.clone(),
            ));
        }
    }

    for column in 0..=gc.column_count {
        for base in 0..gc.base {
            rules.push(Rule::XColumnsEquals(column, base));
        }
    }

    for r in rules.iter() {
        println!("Rule {} bitmask {}", r.formatted(), r.get_mask(gc)?);
    }

    rules.retain(|r| {
        r.get_mask(gc)
            .map(|mask| {
                let ones_count = mask.count_ones();
                let difficulty = ones_count * 100 / gc.solution_count() as usize;
                ones_count > 0 && difficulty > gc.min_difficulty as usize
            })
            .unwrap_or(false)
    });
    println!(
        "Total rules generated (filtered by difficulty): {}",
        rules.len()
    );

    Ok(rules.into())
}

fn generate_verificators(
    ruleset: &Rules,
    gc: &GameConfiguration,
) -> Result<(Code, Verificators), EnigmindError> {
    let mut verificators_before_cleanup = Vec::new();
    let mut final_bitmask: BitMask<u64> = BitMask::ones(gc.solution_count() as usize);

    println!("Picking rules until a single solution is found");
    //While more than one solution
    while final_bitmask.count_ones() > 1 {
        let rule = ruleset.choose(&mut rand::thread_rng()).unwrap();
        let rule_bitmask = rule.get_mask(gc)?;
        let bitmask_and = &final_bitmask & &rule_bitmask;

        let msg;
        if bitmask_and.count_ones() == 0 {
            msg = "skipped (0 sols).".to_string();
        } else if bitmask_and == final_bitmask {
            msg = "skipped (0 impr).".to_string();
        } else {
            verificators_before_cleanup.push(Verifier {
                rule: rule.clone(),
                mask: rule_bitmask.clone(),
            });

            final_bitmask = bitmask_and;
            msg = "chosen.".to_string();
        }
        println!(
            "{} {} Remaining bitmask : {} ({})",
            rule.formatted(),
            msg.pad_to_width(18),
            final_bitmask,
            rule_bitmask.count_ones()
        );
    }

    println!(
        "Total number of rules generated : {}",
        verificators_before_cleanup.len()
    );

    verificators_before_cleanup.sort_by_key(|v| v.mask.count_ones());
    verificators_before_cleanup.reverse();

    let mut final_verificators = verificators_before_cleanup.clone();
    final_verificators.retain(|v| {
        let verificator_bitmask = v.mask.clone();
        let mut other_bitmask = BitMask::ones(gc.solution_count() as usize);
        for other_verificator in &verificators_before_cleanup {
            if *other_verificator != *v {
                //println!("\tAgainst {}", other_verificator);
                other_bitmask &= &other_verificator.mask;
            }
        }
        let is_rule_useful = &verificator_bitmask | &other_bitmask != verificator_bitmask;
        if !is_rule_useful {
            verificators_before_cleanup.retain(|it| it != v);
        }
        is_rule_useful
    });

    let code = Code::from_shift(final_bitmask.trailing_zeros() as u32, gc);
    Ok((code, final_verificators.into()))
}

fn generate_criterias(
    _rules: &Rules,
    verificators: &Verificators,
    gc: &GameConfiguration,
) -> Vec<Criteria> {
    let mut criterias = Vec::new();
    for verif in verificators.deref() {
        let sim_rules = verif.rule.get_similar(gc);
        let (description, rules) = sim_rules.choose(&mut rand::thread_rng()).unwrap();

        criterias.push(Criteria {
            verif: verif.clone(),
            description: description.clone(),
            rules: rules.clone(),
        });
    }
    criterias
}

pub fn generate_game(
    base: u8,
    column_count: u8,
    difficulty_pct: u8,
) -> Result<Game, EnigmindError> {
    let gc = generate_game_configuration(base, column_count, difficulty_pct);
    let rules = generate_rules(&gc)?;

    println!(
        "Rules generated from configuration {:?}: {}\n{}",
        gc,
        rules.len(),
        rules.formatted()
    );

    //pick rules randomly and generate according verificators
    let (code, verificators) = generate_verificators(&rules, &gc)?;

    let sum_complexity: u32 = verificators
        .iter()
        .map(|x| x.mask.count_ones() as u32)
        .sum();
    let mean_complexity = sum_complexity / verificators.len() as u32;
    println!();
    println!(
        "Set of final {} rules (complexity : {}) used to give the unique answer {}:\n{}",
        verificators.len(),
        mean_complexity,
        code,
        verificators.formatted()
    );

    let mut final_mask = BitMask::ones(gc.solution_count() as usize);
    for v in verificators.deref() {
        final_mask &= &v.mask;
    }

    //generate criterias from verificatorset with rules from ruleset
    let criterias = generate_criterias(&rules, &verificators, &gc);

    for crit in &criterias {
        println!("Criteria chosen for {}", crit.verif.rule.formatted());
        println!("\"{}\"", crit.description);
        println!("{}", crit.rules.formatted());
    }

    //generate game object from criterias, secret code and game configuration
    Ok(Game {
        configuration: gc,
        criterias: criterias.into(),
        code,
    })
}

#[cfg(test)]
mod tests {
    use super::GameConfiguration;

    #[test]
    fn test_combination() {
        let gc = GameConfiguration {
            column_count: 3,
            base: 5,
            min_difficulty: 0,
        };

        assert_eq!(gc.get_column_combinations(2).len(), 3);
    }
}
