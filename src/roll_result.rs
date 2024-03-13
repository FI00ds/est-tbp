use std::collections::HashMap;
use std::ops::Deref;
use itertools::Itertools;
use crate::RelicStat;

#[derive(Debug)]
pub struct RollResult(pub Vec<RelicStat>);

impl RollResult {
    pub fn score(&self, weights: &HashMap<RelicStat, f64>) -> f64 {
        // Normalize to crit value and assume mid roll
        6.48 * 0.9 * self.iter()
            .map(|r| weights.get(r).unwrap_or(&0f64))
            .sum::<f64>()
    }

    // p_sub
    pub fn probability(&self, main_stat: RelicStat) -> f64 {
        self.line_probability()
            * self.initial_subs_probability(main_stat)
            * self.upgrade_probability()
    }

    // p_l
    pub fn line_probability(&self) -> f64 {
        if self.len() == 9 {
            0.20
        } else {
            0.80
        }
    }

    // p_t
    pub fn initial_subs_probability(&self, main_stat: RelicStat) -> f64 {
        self.iter()
            .take(4)
            .permutations(4)
            .map(|perm| {
                perm.iter().fold(
                    (100.0 - main_stat.substat_probability_weight() as f64, 1f64),
                    |(remaining_weight, product), r| {
                        let weight = r.substat_probability_weight() as f64;
                        (remaining_weight - weight, product * weight / remaining_weight)
                    },
                ).1
            })
            .sum::<f64>()
    }

    // p_u
    pub fn upgrade_probability(&self) -> f64 {
        // Assumes the upgrade probability is uniform

        let factorial = |n| (2..=n).product::<usize>() as f64;
        let binom = |n, k| factorial(n) / (factorial(k) * factorial(n - k));

        let k = self.len() - 4; // number of upgrades
        let n = 4;

        1.0 / binom(n + k - 1, k)
    }
}

impl Deref for RollResult {
    type Target = Vec<RelicStat>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RollResultIterator(Box<dyn Iterator<Item=RollResult>>);

impl Deref for RollResultIterator {
    type Target = dyn Iterator<Item=RollResult>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RollResultIterator {
    pub fn new(main_stat: RelicStat, num_upgrades: usize) -> Self {
        RollResultIterator(Box::new(
            RelicStat::possible_sub_stats()
                .filter(move |sub| main_stat != *sub)
                .combinations(4)
                .cartesian_product((0u8..4u8).combinations_with_replacement(num_upgrades))
                .map(|(initial, upgrades)| {
                    RollResult(
                        initial.clone().into_iter()
                            .chain(upgrades.into_iter().map(|i| initial[i as usize]))
                            .collect()
                    )
                })
        ))
    }
}

impl Iterator for RollResultIterator {
    type Item = RollResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_subs() {
        // "0 upgrades" should list all possible initial substat combinations (binomial coefficient)
        {
            assert_eq!(RollResultIterator::new(RelicStat::Hp, 0).count(), 330); // 11 choose 4
            assert_eq!(RollResultIterator::new(RelicStat::PhysDmgBoost, 0).count(), 495); // 12 choose 4
        }

        // Initial substat probability sum of all results should add up to 1
        {
            assert_float_eq(
                1.0,
                RollResultIterator::new(RelicStat::Hp, 0)
                    .map(|r| r.initial_subs_probability(RelicStat::Hp))
                    .sum::<f64>(),
            );
        }

        // Initial and upgrade probability should add up to 1
        {
            assert_float_eq(
                1.0,
                RollResultIterator::new(RelicStat::Hp, 4)
                    .map(|r| r.initial_subs_probability(RelicStat::Hp) * r.upgrade_probability())
                    .sum::<f64>(),
            );
        }
    }

    fn assert_float_eq(a: f64, b: f64) {
        let epsilon = 0.00001;
        assert!(epsilon > (a - b).abs())
    }
}