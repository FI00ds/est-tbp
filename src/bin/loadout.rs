use std::collections::HashMap;
use est_tbp::{Relic, ConditionalRelicProbabilityCalculator, RelicSlot, RelicStat};

fn main() {
    let mut weights = HashMap::new();
    weights.insert(RelicStat::CritRate, 1.0);
    weights.insert(RelicStat::CritDmg, 1.0);
    weights.insert(RelicStat::Spd, 1.0);
    weights.insert(RelicStat::AtkPercent, 0.75);
    weights.insert(RelicStat::Atk, 0.25);

    let loadout = {
        use RelicStat::*;
        use RelicSlot::*;

        vec![
            Relic {
                rarity: 5,
                slot: Head,
                main: Hp,
                subs: vec![Def, AtkPercent, Spd, CritDmg, AtkPercent, Spd, Spd, Spd, CritDmg],
            },
            Relic {
                rarity: 5,
                slot: Hands,
                main: Atk,
                subs: vec![Def, AtkPercent, CritRate, CritDmg, CritRate, CritRate, Def, Def],
            },
            Relic {
                rarity: 5,
                slot: Body,
                main: CritRate,
                subs: vec![Hp, Atk, AtkPercent, BreakEffect, Atk, AtkPercent, AtkPercent, BreakEffect],
            },
            Relic {
                rarity: 5,
                slot: Feet,
                main: Spd,
                subs: vec![HpPercent, AtkPercent, EffectHitRate, EffectRes, AtkPercent, AtkPercent, AtkPercent, AtkPercent, AtkPercent],
            },
        ]
    };

    let calculator = ConditionalRelicProbabilityCalculator::new()
        .consider_set()
        .consider_slot()
        .consider_main();

    let mut total_p = 1.0;
    for relic in loadout {
        let score = relic_score(&relic, &weights);
        let p = calculator.calculate_for_relic(&relic, |r| relic_score(r, &weights) >= score);
        total_p *= 1.0 - p;
        println!("{relic:?}");
        print_tbp(p);
        println!();
    }

    println!("overall to improve a single piece:");
    print_tbp(1.0 - total_p);
}

fn relic_score(relic: &Relic, weights: &HashMap<RelicStat, f64>) -> f64 {
    relic.subs.iter()
        .map(|r| weights.get(r).unwrap_or(&0f64))
        .sum::<f64>()
}

fn print_tbp(p: f64) {
    let percent = p * 100.0;
    let relics = 1.0 / p;
    let tbp = relics * 40.0 / 2.1;
    let days = tbp / 240.0;
    println!("{percent:.4}% (1/{relics:.1}), {tbp:.0} tbp ({days:.1}d)");
}