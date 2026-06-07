//! Steady-state / equilibrium detection for rhythmic energy.

use crate::energy::EnsembleEnergy;

/// Criteria for equilibrium detection.
#[derive(Debug, Clone)]
pub struct EquilibriumCriteria {
    /// Maximum relative change in kinetic energy between checks.
    pub kinetic_threshold: f64,
    /// Maximum relative change in harmonic energy between checks.
    pub harmonic_threshold: f64,
    /// Number of consecutive stable checks required.
    pub stability_window: usize,
}

impl Default for EquilibriumCriteria {
    fn default() -> Self {
        Self {
            kinetic_threshold: 0.01,
            harmonic_threshold: 0.01,
            stability_window: 3,
        }
    }
}

/// State of equilibrium detection.
#[derive(Debug, Clone)]
pub struct EquilibriumDetector {
    pub criteria: EquilibriumCriteria,
    /// History of ensemble snapshots.
    pub history: Vec<EnsembleEnergy>,
    /// Count of consecutive stable checks.
    pub stable_count: usize,
}

impl EquilibriumDetector {
    /// Create a new detector with given criteria.
    pub fn new(criteria: EquilibriumCriteria) -> Self {
        Self {
            criteria,
            history: Vec::new(),
            stable_count: 0,
        }
    }

    /// Record a new ensemble snapshot.
    pub fn observe(&mut self, ensemble: &EnsembleEnergy) {
        self.history.push(ensemble.clone());
        if self.history.len() >= 2 {
            let prev = &self.history[self.history.len() - 2];
            let curr = &self.history[self.history.len() - 1];
            if self.is_step_stable(prev, curr) {
                self.stable_count += 1;
            } else {
                self.stable_count = 0;
            }
        }
    }

    /// Check if a single step is stable.
    fn is_step_stable(&self, prev: &EnsembleEnergy, curr: &EnsembleEnergy) -> bool {
        if prev.len() != curr.len() {
            return false;
        }
        for (a, b) in prev.agents.iter().zip(curr.agents.iter()) {
            if a.agent_id != b.agent_id {
                return false;
            }
            let kinetic_change = if a.kinetic == 0.0 {
                b.kinetic
            } else {
                ((b.kinetic - a.kinetic) / a.kinetic).abs()
            };
            let harmonic_change = if a.harmonic == 0.0 {
                b.harmonic
            } else {
                ((b.harmonic - a.harmonic) / a.harmonic).abs()
            };
            if kinetic_change > self.criteria.kinetic_threshold
                || harmonic_change > self.criteria.harmonic_threshold
            {
                return false;
            }
        }
        true
    }

    /// Whether the ensemble has reached equilibrium.
    pub fn is_equilibrium(&self) -> bool {
        self.stable_count >= self.criteria.stability_window
    }

    /// Measure how close the ensemble is to equilibrium (0.0 = far, 1.0 = at equilibrium).
    pub fn equilibrium_progress(&self) -> f64 {
        if self.criteria.stability_window == 0 {
            return 1.0;
        }
        (self.stable_count as f64 / self.criteria.stability_window as f64).min(1.0)
    }

    /// Reset the detector.
    pub fn reset(&mut self) {
        self.history.clear();
        self.stable_count = 0;
    }

    /// Check if the ensemble kinetic/harmonic ratio has converged to a target.
    pub fn is_ratio_converged(
        &self,
        target_ratio: f64,
        tolerance: f64,
    ) -> bool {
        if let Some(latest) = self.history.last() {
            let actual = latest.average_kinetic_ratio();
            (actual - target_ratio).abs() <= tolerance
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::RhythmicEnergy;

    #[test]
    fn test_equilibrium_detected() {
        let criteria = EquilibriumCriteria {
            kinetic_threshold: 0.1,
            harmonic_threshold: 0.1,
            stability_window: 3,
        };
        let mut detector = EquilibriumDetector::new(criteria);
        let ens = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
        for _ in 0..5 {
            detector.observe(&ens);
        }
        assert!(detector.is_equilibrium());
    }

    #[test]
    fn test_equilibrium_not_detected_with_change() {
        let criteria = EquilibriumCriteria {
            kinetic_threshold: 0.01,
            harmonic_threshold: 0.01,
            stability_window: 3,
        };
        let mut detector = EquilibriumDetector::new(criteria);
        for i in 0..5 {
            let ens = EnsembleEnergy::new(vec![
                RhythmicEnergy::new(0, 5.0 + i as f64, 5.0),
            ]);
            detector.observe(&ens);
        }
        assert!(!detector.is_equilibrium());
    }

    #[test]
    fn test_progress_tracking() {
        let criteria = EquilibriumCriteria {
            kinetic_threshold: 0.1,
            harmonic_threshold: 0.1,
            stability_window: 4,
        };
        let mut detector = EquilibriumDetector::new(criteria);
        let ens = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
        for _ in 0..3 {
            detector.observe(&ens);
        }
        // 2 stable counts out of 4 needed (first observe has no previous)
        assert!(!detector.is_equilibrium());
        assert!(detector.equilibrium_progress() > 0.0);
    }

    #[test]
    fn test_ratio_converged() {
        let criteria = EquilibriumCriteria::default();
        let mut detector = EquilibriumDetector::new(criteria);
        let ens = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
        detector.observe(&ens);
        assert!(detector.is_ratio_converged(0.5, 0.01));
    }

    #[test]
    fn test_reset() {
        let criteria = EquilibriumCriteria::default();
        let mut detector = EquilibriumDetector::new(criteria);
        detector.observe(&EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]));
        detector.reset();
        assert!(detector.history.is_empty());
        assert_eq!(detector.stable_count, 0);
    }
}
