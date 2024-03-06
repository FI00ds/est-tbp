use std::array::IntoIter;
use crate::roll_result::{RollResult, RollResultIterator};

pub mod roll_result;

pub fn p_main(slot: RelicSlot, main_stat: RelicStat) -> f64 {
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

pub fn p_sub(main_stat: RelicStat, filter: impl Fn(&RollResult) -> bool) -> f64 {
    RollResultIterator::new(main_stat, 4)
        .chain(RollResultIterator::new(main_stat, 5))
        .filter(filter)
        .map(|r| r.probability(main_stat))
        .sum::<f64>()
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

#[derive(Debug)]
pub enum RelicSlot {
    Head,
    Hands,
    Body,
    Feet,
    Orb,
    Rope,
}