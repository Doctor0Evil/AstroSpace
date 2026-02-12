#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FearRegime {
    Low,    // Reckless, pain denial
    Tuned,  // Teachable discomfort
    Hyper,  // Paralysis, over-avoidance
}

impl FearRegime {
    pub fn from_overload_streak(streak: u32) -> Self {
        if streak < 3 { Self::Low }
        else if streak < 10 { Self::Tuned }
        else { Self::Hyper }
    }
}

#[derive(Debug)]
pub struct PainMetrics {
    pub overload_freq: f64,     // [0,1]
    pub trust_drop_avg: f64,    // [0,1]
    pub repair_necessity: f64,  // [0,1]
    pub regime: FearRegime,
}

impl PainMetrics {
    pub fn compute_from_lattice(lattice: &super::Lattice) -> Self {
        let mut overload_count = 0.0;
        let mut trust_drops = 0.0;
        let mut repair_need = 0.0;
        let n = lattice.sites.len() as f64;

        for site in &lattice.sites {
            if site.bioload > 0.7 { overload_count += 1.0; }
            trust_drops += (1.0 - site.trust).max(0.0);
            repair_need += site.fear * site.bioload;
        }

        let overload_freq = overload_count / n;
        let trust_drop_avg = trust_drops / n;
        let repair_necessity = repair_need / n;
        let streak = (overload_count as u32).min(20);  // Proxy streak
        let regime = FearRegime::from_overload_streak(streak);

        Self {
            overload_freq: overload_freq.clamp(0.0, 1.0),
            trust_drop_avg: trust_drop_avg.clamp(0.0, 1.0),
            repair_necessity: repair_necessity.clamp(0.0, 1.0),
            regime,
        }
    }

    pub fn is_teachable(&self) -> bool {
        matches!(self.regime, FearRegime::Tuned) && self.repair_necessity > 0.3 && self.overload_freq < 0.7
    }
}
