//! Conservation law validation: γ + H = C.

use crate::energy::{EnsembleEnergy, RhythmicEnergy};

/// Result of conservation law check.
#[derive(Debug, Clone)]
pub struct ConservationResult {
    /// Initial total energy.
    pub initial_total: f64,
    /// Current total energy.
    pub current_total: f64,
    /// Whether conservation holds within tolerance.
    pub is_conserved: bool,
    /// Deviation from conservation.
    pub deviation: f64,
}

/// Conservation law validator.
pub struct ConservationLaw {
    /// Conserved total energy constant C.
    pub constant_c: f64,
    /// Tolerance for floating-point comparison.
    pub tolerance: f64,
}

impl ConservationLaw {
    /// Create a new conservation law with given constant and tolerance.
    pub fn new(constant_c: f64, tolerance: f64) -> Self {
        Self { constant_c, tolerance }
    }

    /// Derive the conservation constant from an ensemble.
    pub fn from_ensemble(ensemble: &EnsembleEnergy, tolerance: f64) -> Self {
        Self {
            constant_c: ensemble.total(),
            tolerance,
        }
    }

    /// Check if a single agent's energy is conserved.
    pub fn check_agent(&self, agent: &RhythmicEnergy) -> ConservationResult {
        let current = agent.total();
        let deviation = (current - self.constant_c).abs();
        ConservationResult {
            initial_total: self.constant_c,
            current_total: current,
            is_conserved: deviation <= self.tolerance,
            deviation,
        }
    }

    /// Check if the ensemble total is conserved.
    pub fn check_ensemble(&self, ensemble: &EnsembleEnergy) -> ConservationResult {
        let current = ensemble.total();
        let deviation = (current - self.constant_c).abs();
        ConservationResult {
            initial_total: self.constant_c,
            current_total: current,
            is_conserved: deviation <= self.tolerance,
            deviation,
        }
    }

    /// Verify that per-agent energy is individually conserved.
    /// (Each agent has its own C from when it was initialized.)
    pub fn check_per_agent(agent_constants: &[(usize, f64)], agents: &[RhythmicEnergy], tolerance: f64) -> Vec<ConservationResult> {
        agent_constants.iter().map(|(id, c)| {
            if let Some(agent) = agents.iter().find(|a| a.agent_id == *id) {
                let current = agent.total();
                let deviation = (current - c).abs();
                ConservationResult {
                    initial_total: *c,
                    current_total: current,
                    is_conserved: deviation <= tolerance,
                    deviation,
                }
            } else {
                ConservationResult {
                    initial_total: *c,
                    current_total: 0.0,
                    is_conserved: false,
                    deviation: *c,
                }
            }
        }).collect()
    }

    /// Normalize an ensemble to satisfy conservation by rescaling.
    pub fn enforce_ensemble(&self, ensemble: &mut EnsembleEnergy) {
        let current = ensemble.total();
        if current == 0.0 {
            return;
        }
        let scale = self.constant_c / current;
        ensemble.agents.iter_mut().for_each(|a| {
            a.kinetic *= scale;
            a.harmonic *= scale;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservation_holds() {
        let ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
        let law = ConservationLaw::from_ensemble(&ensemble, 1e-10);
        let result = law.check_ensemble(&ensemble);
        assert!(result.is_conserved);
    }

    #[test]
    fn test_conservation_violation_detected() {
        let law = ConservationLaw::new(10.0, 0.01);
        let ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 6.0, 5.0),
        ]);
        let result = law.check_ensemble(&ensemble);
        assert!(!result.is_conserved);
    }

    #[test]
    fn test_enforce_conservation() {
        let law = ConservationLaw::new(10.0, 1e-10);
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 6.0, 6.0),
        ]);
        law.enforce_ensemble(&mut ensemble);
        assert!((ensemble.total() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_per_agent_check() {
        let constants = vec![(0, 10.0), (1, 8.0)];
        let agents = vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
            RhythmicEnergy::new(1, 4.0, 6.0),
        ];
        let results = ConservationLaw::check_per_agent(&constants, &agents, 1e-10);
        assert!(results[0].is_conserved);
        assert!(!results[1].is_conserved);
    }
}
