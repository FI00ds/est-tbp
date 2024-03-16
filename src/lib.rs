use std::array::IntoIter;

use itertools::Itertools;

pub use probability::ConditionalRelicProbabilityCalculator;

mod probability;

#[derive(Clone, Debug)]
pub struct Relic {
    pub rarity: usize,
    pub slot: RelicSlot,
    pub main: RelicStat,
    pub subs: Vec<RelicStat>,
}

impl Relic {
    pub fn new(rarity: usize, slot: RelicSlot, main: RelicStat) -> Self {
        Self { rarity, slot, main, subs: vec![] }
    }

    pub fn p_main(&self) -> f64 {
        self.p_main_set() * self.p_main_slot() * self.p_main_stat()
    }

    pub fn p_main_set(&self) -> f64 {
        1.0 / 2.0
    }

    pub fn p_main_slot(&self) -> f64 {
        use RelicSlot::*;
        match self.slot {
            Head | Hands | Body | Feet => 0.25,
            Orb | Rope => 0.5
        }
    }

    pub fn p_main_stat(&self) -> f64 {
        use RelicSlot::*;
        use RelicStat::*;
        match self.slot {
            Head | Hands => 1.0,
            Body => match self.main {
                HpPercent | AtkPercent | DefPercent => 0.2,
                CritRate | CritDmg | HealingBoost | EffectHitRate => 0.1,
                _ => 0.0,
            }
            Feet => match self.main {
                HpPercent | AtkPercent | DefPercent => 0.3,
                Spd => 0.1,
                _ => 0.0
            }
            Orb => match self.main {
                HpPercent | AtkPercent | DefPercent => 0.12,

                PhysDmgBoost | FireDmgBoost | IceDmgBoost |
                WindDmgBoost | LightningDmgBoost | QuantumDmgBoost |
                ImaginaryDmgBoost => 0.65 / 7.0,

                _ => 0.0
            }
            Rope => match self.main {
                HpPercent | AtkPercent | DefPercent => 0.8 / 3.0,
                BreakEffect => 0.15,
                EnergyRegenRate => 0.05,
                _ => 0.0
            }
        }
    }

    pub fn p_sub(&self) -> f64 {
        self.p_sub_line() * self.p_sub_i() * self.p_sub_u()
    }

    pub fn p_sub_line(&self) -> f64 {
        let max = self.rarity.saturating_sub(1) + self.rarity;
        if self.subs.len() == max {
            0.20
        } else {
            0.80
        }
    }

    pub fn p_sub_i(&self) -> f64 {
        let remaining_weight = 100.0 - self.main.substat_probability_weight() as f64;

        self.subs.iter()
            .take(4)
            .permutations(self.subs.len().min(4))
            .map(|perm| {
                perm.iter().fold(
                    (remaining_weight, 1f64),
                    |(remaining_weight, product), r| {
                        let weight = r.substat_probability_weight() as f64;
                        (remaining_weight - weight, product * weight / remaining_weight)
                    },
                ).1
            })
            .sum::<f64>()
    }

    pub fn p_sub_u(&self) -> f64 {
        // Assumes the upgrade probability is uniform
        let factorial = |n| (2..=n).product::<usize>() as f64;
        let binom = |n, k| factorial(n) / (factorial(k) * factorial(n - k));

        let k = self.subs.len().saturating_sub(4); // number of upgrades
        let n = 4;

        1.0 / binom(n + k - 1, k)
    }

    pub fn copy_with_new_subs(&self, subs: Vec<RelicStat>) -> Self {
        Self {
            rarity: self.rarity,
            slot: self.slot,
            main: self.main,
            subs,
        }
    }

    pub fn filtered_p_sub(&self, filter: impl FnMut(&Relic) -> bool) -> f64 {
        SubstatIterator::new_from_relic(self)
            .map(|subs| self.copy_with_new_subs(subs))
            .filter(filter)
            .map(|r| r.p_sub())
            .sum()
    }
}


#[derive(PartialEq, Eq, Hash, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum RelicStat {
    Hp,
    Atk,
    Def,
    HpPercent,
    AtkPercent,
    DefPercent,
    Spd,
    CritRate,
    CritDmg,
    EffectHitRate,
    EffectRes,
    BreakEffect,
    // Main stat only
    EnergyRegenRate,
    HealingBoost,
    PhysDmgBoost,
    FireDmgBoost,
    IceDmgBoost,
    WindDmgBoost,
    LightningDmgBoost,
    QuantumDmgBoost,
    ImaginaryDmgBoost,
}

impl RelicStat {
    pub fn possible_sub_stats() -> IntoIter<RelicStat, 12> {
        use RelicStat::*;
        [
            Hp,
            Atk,
            Def,
            HpPercent,
            AtkPercent,
            DefPercent,
            Spd,
            CritRate,
            CritDmg,
            EffectHitRate,
            EffectRes,
            BreakEffect,
        ].into_iter()
    }

    // source: https://docs.qq.com/sheet/DYkFxSVFNSGp5YlVv?tab=metuhj
    pub fn substat_probability_weight(&self) -> u8 {
        use RelicStat::*;
        match self {
            Hp => 10,
            Atk => 10,
            Def => 10,
            HpPercent => 10,
            AtkPercent => 10,
            DefPercent => 10,
            Spd => 4,
            CritRate => 6,
            CritDmg => 6,
            EffectHitRate => 8,
            EffectRes => 8,
            BreakEffect => 8,
            _ => 0
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelicSlot {
    Head,
    Hands,
    Body,
    Feet,
    Orb,
    Rope,
}

struct SubstatIterator(Box<dyn Iterator<Item=Vec<RelicStat>>>);

impl SubstatIterator {
    pub fn new_from_relic(relic: &Relic) -> SubstatIterator {
        // e.g. a 5* relic can start with either 3 or 4 initial substats
        let max_initial = relic.rarity.saturating_sub(1);
        let min_initial = max_initial.saturating_sub(1);

        let relic = relic.clone();

        SubstatIterator(Box::new(
            // generate both 3-liners and 4 liners
            (min_initial..=max_initial)
                .flat_map(move |initial| Self::with_params(relic.main, initial, relic.rarity))
        ))
    }

    fn with_params(main: RelicStat, initial: usize, num_upgrades: usize) -> SubstatIterator {
        let fill_initial = (initial + num_upgrades).min(4);
        let fill_upgrades = num_upgrades.saturating_sub(fill_initial - initial);

        SubstatIterator(Box::new(
            // exclude main stat in substat
            RelicStat::possible_sub_stats().filter(move |sub| main != *sub)
                // generate a set of 4 initial substats
                .combinations(fill_initial)
                // generate a set of upgrades represented by indices on the 4 initial substats
                .cartesian_product(
                    (0..4).combinations_with_replacement(fill_upgrades)
                )
                // join into a single vector
                .map(|(initial, upgrades)| {
                    initial.clone().into_iter()
                        .chain(upgrades.into_iter().map(|i| initial[i]))
                        .collect::<Vec<RelicStat>>()
                })
        ))
    }
}

impl Iterator for SubstatIterator {
    type Item = Vec<RelicStat>;

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
            assert_eq!(
                SubstatIterator::with_params(RelicStat::Hp, 4, 0).count(),
                330 // 11 choose 4
            );
            assert_eq!(
                SubstatIterator::with_params(RelicStat::PhysDmgBoost, 4, 0).count(),
                495 // 12 choose 4
            );
        }

        // Initial substat probability sum of all results should add up to 1
        {
            let relic = Relic {
                rarity: 5,
                slot: RelicSlot::Head,
                main: RelicStat::Hp,
                subs: Vec::new(),
            };

            assert_float_eq(
                1.0,
                SubstatIterator::with_params(RelicStat::Hp, 4, 0)
                    .map(|subs| relic.copy_with_new_subs(subs).p_sub_i())
                    .sum::<f64>(),
            );
        }

        // Initial and upgrade probability should add up to 1
        {
            let relic = Relic {
                rarity: 5,
                slot: RelicSlot::Head,
                main: RelicStat::Hp,
                subs: Vec::new(),
            };

            assert_float_eq(
                1.0,
                SubstatIterator::with_params(RelicStat::Hp, 4, 5)
                    .map(|subs| relic.copy_with_new_subs(subs))
                    .map(|r| r.p_sub_i() * r.p_sub_u())
                    .sum::<f64>(),
            );
        }

        // chain implementation is correct
        {
            let relic = Relic {
                rarity: 5,
                slot: RelicSlot::Head,
                main: RelicStat::Hp,
                subs: Vec::new(),
            };

            assert_eq!(
                SubstatIterator::new_from_relic(&relic).count(),
                SubstatIterator::with_params(RelicStat::Hp, 3, 5)
                    .chain(SubstatIterator::with_params(RelicStat::Hp, 4, 5))
                    .count(),
            )
        }

        // Overall probability should add up to 1
        {
            let relic = Relic {
                rarity: 5,
                slot: RelicSlot::Head,
                main: RelicStat::Hp,
                subs: Vec::new(),
            };

            println!("{}", SubstatIterator::new_from_relic(&relic)
                .map(|subs| relic.copy_with_new_subs(subs))
                .map(|r| r.p_sub_line() * r.p_sub_i() * r.p_sub_u())
                .sum::<f64>());

            // SubstatIterator::new_from_relic(&relic).for_each(|s| println!("{s:?}"));

            assert_float_eq(
                1.0,
                SubstatIterator::new_from_relic(&relic)
                    .map(|subs| relic.copy_with_new_subs(subs))
                    .map(|r| r.p_sub_line() * r.p_sub_i() * r.p_sub_u())
                    .sum::<f64>(),
            );
        }
    }

    fn assert_float_eq(a: f64, b: f64) {
        let epsilon = 0.00001;
        assert!(epsilon > (a - b).abs())
    }
}