use est_tbp::{Relic, /* ConditionalRelicProbabilityCalculator,  */RelicSlot, RelicStat};

fn main() {
    let relic = Relic::new(5, RelicSlot::Head, RelicStat::Hp);
    for i in 1..=7 {
        calculate(&relic, i);
    }
}

fn calculate(relic: &Relic, crit_rolls: usize) {
    let filter = |r: &Relic| r.subs.iter()
        .filter(|r| matches!(r, RelicStat::CritRate | RelicStat::CritDmg))
        .count()
        >= crit_rolls;

    let crit_rolls = crit_rolls as f64;

    println!("=====================================================");
    println!(
        "params: {crit_rolls:?} crit rolls, {:.1}~{:.1} CV",
        crit_rolls * 0.8 * 6.48, crit_rolls * 6.48
    );

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