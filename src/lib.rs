use std::array::IntoIter;

pub mod roll_result;

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