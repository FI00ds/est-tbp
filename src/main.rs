use std::collections::HashMap;
use est_tbp::{RelicSlot, RelicStat};
use est_tbp::roll_result::{RollResult, RollResultIterator};

fn main() {
    let mut score_weights = HashMap::new();
    score_weights.insert(RelicStat::CritRate, 1.0);
    score_weights.insert(RelicStat::CritDmg, 1.0);
    score_weights.insert(RelicStat::Spd, 1.0);
    score_weights.insert(RelicStat::Atk, 0.75);

    let filter = |r: &_| substat_score(r, &score_weights) > 30.0;

    calculate(RelicSlot::Head, RelicStat::Hp, filter);
    calculate(RelicSlot::Hands, RelicStat::Atk, filter);
    calculate(RelicSlot::Body, RelicStat::CritRate, filter);
    calculate(RelicSlot::Feet, RelicStat::Spd, filter);
    calculate(RelicSlot::Orb, RelicStat::IceDmgBoost, filter);
    calculate(RelicSlot::Rope, RelicStat::AtkPercent, filter);
}

fn calculate(slot: RelicSlot, main_stat: RelicStat, filter: impl Fn(&RollResult) -> bool) {
    println!("=====================================================");
    println!("params: {slot:?}, {main_stat:?}");

    let p_main = p_main(slot, main_stat);
    let p_sub = p_sub(main_stat, filter);
    let p = p_main * p_sub;

    println!("p_main = {:.2}%", p_main * 100.0);
    println!("p_sub  = {:.2}%", p_sub * 100.0);
    println!("p      = {:.2}%", p * 100.0);

    let est_relic_count = 1.0 / p;
    let tbp_per_relic = 40.0 / 2.1;
    let est_tbp = est_relic_count * tbp_per_relic;

    println!();
    println!("est. relic count = {:.1}", est_relic_count);
    println!("        est. tbp = {:.1} ({:.1} days)", est_tbp, est_tbp/240.0);
}

fn p_main(slot: RelicSlot, main_stat: RelicStat) -> f64 {
    use RelicSlot::*;
    use RelicStat::*;

    let p_set = 0.5;
    let p_slot = match slot {
        Head | Hands | Body | Feet => 0.25,
        Orb | Rope => 0.5
    };

    // source: https://docs.qq.com/sheet/DYkFxSVFNSGp5YlVv?tab=metuhj
    let p_main = match slot {
        Head | Hands => 1.0,
        Body => match main_stat {
            HpPercent | AtkPercent | DefPercent => 0.2,
            CritRate | CritDmg | HealingBoost | EffectHitRate => 0.1,
            _ => 0.0,
        }
        Feet => match main_stat {
            HpPercent | AtkPercent | DefPercent => 0.3,
            Spd => 0.1,
            _ => 0.0
        }
        Orb => match main_stat {
            HpPercent | AtkPercent | DefPercent => 0.12,

            PhysDmgBoost | FireDmgBoost | IceDmgBoost |
            WindDmgBoost | LightningDmgBoost | QuantumDmgBoost |
            ImaginaryDmgBoost => 0.65 / 7.0,

            _ => 0.0
        }
        Rope => match main_stat {
            HpPercent | AtkPercent | DefPercent => 0.8 / 3.0,
            BreakEffect => 0.15,
            EnergyRegenRate => 0.05,
            _ => 0.0
        }
    };

    p_set * p_slot * p_main
}

fn p_sub(main_stat: RelicStat, filter: impl Fn(&RollResult) -> bool) -> f64 {
    RollResultIterator::new(main_stat, 4)
        .chain(RollResultIterator::new(main_stat, 5))
        .filter(filter)
        .map(|r| {
            let line_probability = if r.len() == 9 {
                0.20
            } else {
                0.80
            };

            line_probability * r.initial_subs_probability(main_stat) * r.upgrade_probability()
        }).sum::<f64>()
}

fn substat_score(r: &RollResult, weights: &HashMap<RelicStat, f64>) -> f64 {
    // scale to CV and assume mid-rolls
    6.48 * 0.9 * r.iter()
        .map(|r| weights.get(r).unwrap_or(&0f64))
        .sum::<f64>()
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