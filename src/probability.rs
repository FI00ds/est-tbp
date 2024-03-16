use crate::Relic;

#[derive(Debug, Clone, Default)]
pub struct ConditionalRelicProbabilityCalculator {
    consider_set: bool,
    consider_slot: bool,
    consider_main: bool,
}

impl ConditionalRelicProbabilityCalculator {
    pub fn new() -> Self { Self::default() }
    pub fn consider_set(mut self) -> Self {
        self.consider_set = true;
        self
    }
    pub fn consider_slot(mut self) -> Self {
        self.consider_slot = true;
        self
    }
    pub fn consider_main(mut self) -> Self {
        self.consider_main = true;
        self
    }

    pub fn calculate_for_relic(&self, relic: &Relic, filter: impl FnMut(&Relic) -> bool) -> f64 {
        let mut p = relic.filtered_p_sub(filter);

        if self.consider_set {
            p *= relic.p_main_set();
        }

        if self.consider_slot {
            p *= relic.p_main_slot();
        }

        if self.consider_main {
            p *= relic.p_main_stat();
        }
        
        p
    }
}