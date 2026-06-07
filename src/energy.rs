//! Rhythmic energy types: kinetic (γ) and harmonic potential (H).

/// Rhythmic energy vector for a single agent.
#[derive(Debug, Clone, PartialEq)]
pub struct RhythmicEnergy {
    /// Agent identifier.
    pub agent_id: usize,
    /// Kinetic rhythmic energy (γ): active, driving energy.
    pub kinetic: f64,
    /// Harmonic potential energy (H): stored, latent energy.
    pub harmonic: f64,
}

impl RhythmicEnergy {
    /// Create a new rhythmic energy with given kinetic and harmonic components.
    pub fn new(agent_id: usize, kinetic: f64, harmonic: f64) -> Self {
        Self { agent_id, kinetic, harmonic }
    }

    /// Total energy: γ + H.
    pub fn total(&self) -> f64 {
        self.kinetic + self.harmonic
    }

    /// Convert entirely to kinetic (maximize drive).
    pub fn to_kinetic(&self) -> Self {
        Self {
            agent_id: self.agent_id,
            kinetic: self.total(),
            harmonic: 0.0,
        }
    }

    /// Convert entirely to harmonic (maximize potential).
    pub fn to_harmonic(&self) -> Self {
        Self {
            agent_id: self.agent_id,
            kinetic: 0.0,
            harmonic: self.total(),
        }
    }

    /// Shift energy toward kinetic by a factor in [0, 1].
    pub fn shift_kinetic(&self, factor: f64) -> Self {
        let total = self.total();
        let new_kinetic = self.kinetic + factor * self.harmonic;
        let new_kinetic = new_kinetic.min(total);
        Self {
            agent_id: self.agent_id,
            kinetic: new_kinetic,
            harmonic: total - new_kinetic,
        }
    }

    /// Shift energy toward harmonic by a factor in [0, 1].
    pub fn shift_harmonic(&self, factor: f64) -> Self {
        let total = self.total();
        let new_harmonic = self.harmonic + factor * self.kinetic;
        let new_harmonic = new_harmonic.min(total);
        Self {
            agent_id: self.agent_id,
            kinetic: total - new_harmonic,
            harmonic: new_harmonic,
        }
    }

    /// Ratio of kinetic to total energy.
    pub fn kinetic_ratio(&self) -> f64 {
        let total = self.total();
        if total == 0.0 { 0.5 } else { self.kinetic / total }
    }

    /// Zero energy for an agent.
    pub fn zero(agent_id: usize) -> Self {
        Self::new(agent_id, 0.0, 0.0)
    }

    /// Scale both components uniformly.
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            agent_id: self.agent_id,
            kinetic: self.kinetic * factor,
            harmonic: self.harmonic * factor,
        }
    }
}

/// Ensemble energy: collection of agent energies.
#[derive(Debug, Clone)]
pub struct EnsembleEnergy {
    pub agents: Vec<RhythmicEnergy>,
}

impl EnsembleEnergy {
    /// Create from a vector of agent energies.
    pub fn new(agents: Vec<RhythmicEnergy>) -> Self {
        Self { agents }
    }

    /// Total ensemble energy (sum of all agent totals).
    pub fn total(&self) -> f64 {
        self.agents.iter().map(|a| a.total()).sum()
    }

    /// Total kinetic energy across ensemble.
    pub fn total_kinetic(&self) -> f64 {
        self.agents.iter().map(|a| a.kinetic).sum()
    }

    /// Total harmonic potential across ensemble.
    pub fn total_harmonic(&self) -> f64 {
        self.agents.iter().map(|a| a.harmonic).sum()
    }

    /// Average kinetic ratio across ensemble.
    pub fn average_kinetic_ratio(&self) -> f64 {
        if self.agents.is_empty() {
            return 0.5;
        }
        self.agents.iter().map(|a| a.kinetic_ratio()).sum::<f64>() / self.agents.len() as f64
    }

    /// Get energy for a specific agent.
    pub fn get_agent(&self, agent_id: usize) -> Option<&RhythmicEnergy> {
        self.agents.iter().find(|a| a.agent_id == agent_id)
    }

    /// Update energy for a specific agent.
    pub fn update_agent(&mut self, updated: RhythmicEnergy) {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.agent_id == updated.agent_id) {
            *agent = updated;
        }
    }

    /// Number of agents.
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Whether the ensemble is empty.
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_energy() {
        let e = RhythmicEnergy::new(0, 3.0, 7.0);
        assert_eq!(e.agent_id, 0);
        assert!((e.kinetic - 3.0).abs() < 1e-10);
        assert!((e.harmonic - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_total_energy() {
        let e = RhythmicEnergy::new(1, 4.0, 6.0);
        assert!((e.total() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_to_kinetic() {
        let e = RhythmicEnergy::new(0, 3.0, 7.0).to_kinetic();
        assert!((e.kinetic - 10.0).abs() < 1e-10);
        assert!((e.harmonic).abs() < 1e-10);
    }

    #[test]
    fn test_to_harmonic() {
        let e = RhythmicEnergy::new(0, 3.0, 7.0).to_harmonic();
        assert!((e.kinetic).abs() < 1e-10);
        assert!((e.harmonic - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_shift_kinetic() {
        let e = RhythmicEnergy::new(0, 2.0, 8.0);
        let shifted = e.shift_kinetic(0.5);
        assert!((shifted.kinetic - 6.0).abs() < 1e-10);
        assert!((shifted.harmonic - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_ensemble_total() {
        let ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 3.0, 7.0),
            RhythmicEnergy::new(1, 5.0, 5.0),
        ]);
        assert!((ensemble.total() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_ensemble_total_kinetic() {
        let ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 3.0, 7.0),
            RhythmicEnergy::new(1, 5.0, 5.0),
        ]);
        assert!((ensemble.total_kinetic() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_ensemble_average_kinetic_ratio() {
        let ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 3.0, 7.0),
            RhythmicEnergy::new(1, 5.0, 5.0),
        ]);
        let avg = ensemble.average_kinetic_ratio();
        assert!(avg > 0.3 && avg < 0.7);
    }
}
