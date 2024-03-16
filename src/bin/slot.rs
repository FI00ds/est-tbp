use std::collections::HashMap;

use est_tbp::{Relic, RelicSlot, RelicStat};

fn main() {
    let mut score_weights = HashMap::new();
    score_weights.insert(RelicStat::CritRate, 1.0);
    score_weights.insert(RelicStat::CritDmg, 1.0);
    score_weights.insert(RelicStat::Spd, 1.0);
    score_weights.insert(RelicStat::AtkPercent, 0.75);
    score_weights.insert(RelicStat::Atk, 0.25);

    let filter = |r: &_| relic_score(r, &score_weights) >= 6.0;

    calculate(Relic::new(5, RelicSlot::Head, RelicStat::Hp), filter);
    calculate(Relic::new(5, RelicSlot::Hands, RelicStat::Atk), filter);
    calculate(Relic::new(5, RelicSlot::Body, RelicStat::CritRate), filter);
    calculate(Relic::new(5, RelicSlot::Feet, RelicStat::Spd), filter);
    calculate(Relic::new(5, RelicSlot::Orb, RelicStat::IceDmgBoost), filter);
    calculate(Relic::new(5, RelicSlot::Rope, RelicStat::AtkPercent), filter);
}

fn calculate(relic: Relic, filter: impl Fn(&Relic) -> bool) {
    println!("=====================================================");
    println!("{relic:?}");

    let p_main = relic.p_main();
    let p_sub = relic.filtered_p_sub(filter);
    let p = p_main * p_sub;

    let est_relic_count = 1.0 / p;
    let tbp_per_relic = 40.0 / 2.1;
    let est_tbp = est_relic_count * tbp_per_relic;

    println!("   p_main   = {:>6.3}%   (1/{:.1})", p_main * 100.0, 1.0 / p_main);
    println!("   p_sub    = {:>6.3}%   (1/{:.1})", p_sub * 100.0, 1.0 / p_sub);
    println!("   p        = {:>6.3}%   (1/{:.1})", p * 100.0, 1.0 / p);
    println!("   est. tbp =  {:>6.0}   ({:.1} days)", est_tbp, est_tbp / 240.0);
}

fn relic_score(relic: &Relic, weights: &HashMap<RelicStat, f64>) -> f64 {
    relic.subs.iter()
        .map(|r| weights.get(r).unwrap_or(&0f64))
        .sum::<f64>()
}