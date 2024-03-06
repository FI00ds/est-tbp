use std::collections::HashMap;
use est_tbp::{p_main, p_sub, RelicSlot, RelicStat};
use est_tbp::roll_result::RollResult;

fn main() {
    let mut score_weights = HashMap::new();
    score_weights.insert(RelicStat::CritRate, 1.0);
    score_weights.insert(RelicStat::CritDmg, 1.0);
    score_weights.insert(RelicStat::Spd, 1.0);
    score_weights.insert(RelicStat::Atk, 0.75);

    let filter = |r: &_| substat_weight(r, &score_weights) >= 6.0;

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


    let est_relic_count = 1.0 / p;
    let tbp_per_relic = 40.0 / 2.1;
    let est_tbp = est_relic_count * tbp_per_relic;

    println!("   p_main   = {:>6.3}%   (1/{:.1})", p_main * 100.0, 1.0 / p_main);
    println!("   p_sub    = {:>6.3}%   (1/{:.1})", p_sub * 100.0, 1.0 / p_sub);
    println!("   p        = {:>6.3}%   (1/{:.1})", p * 100.0, 1.0 / p);
    println!("   est. tbp =  {:>6.0}   ({:.1} days)", est_tbp, est_tbp / 240.0);
}

fn substat_weight(r: &RollResult, weights: &HashMap<RelicStat, f64>) -> f64 {
    // scale to CV and assume mid-rolls
    0.9 * r.iter()
        .map(|r| weights.get(r).unwrap_or(&0f64))
        .sum::<f64>()
}