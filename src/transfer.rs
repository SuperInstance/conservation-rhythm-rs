//! Energy transfer between agents while maintaining conservation.

use crate::conservation::ConservationLaw;
use crate::energy::EnsembleEnergy;

/// A record of energy transfer between two agents.
#[derive(Debug, Clone)]
pub struct TransferRecord {
    pub from_agent: usize,
    pub to_agent: usize,
    pub kinetic_transferred: f64,
    pub harmonic_transferred: f64,
}

impl TransferRecord {
    /// Total energy transferred.
    pub fn total_transferred(&self) -> f64 {
        self.kinetic_transferred + self.harmonic_transferred
    }
}

/// Transfer energy between agents in an ensemble.
pub struct EnergyTransfer;

impl EnergyTransfer {
    /// Transfer kinetic energy from one agent to another.
    /// Returns a TransferRecord if successful.
    pub fn transfer_kinetic(
        ensemble: &mut EnsembleEnergy,
        from_id: usize,
        to_id: usize,
        amount: f64,
    ) -> Option<TransferRecord> {
        if from_id == to_id || amount < 0.0 {
            return None;
        }

        let from_idx = ensemble.agents.iter().position(|a| a.agent_id == from_id)?;
        let to_idx = ensemble.agents.iter().position(|a| a.agent_id == to_id)?;

        let actual_amount = amount.min(ensemble.agents[from_idx].kinetic);
        if actual_amount == 0.0 {
            return None;
        }

        ensemble.agents[from_idx].kinetic -= actual_amount;
        ensemble.agents[to_idx].kinetic += actual_amount;

        Some(TransferRecord {
            from_agent: from_id,
            to_agent: to_id,
            kinetic_transferred: actual_amount,
            harmonic_transferred: 0.0,
        })
    }

    /// Transfer harmonic potential energy from one agent to another.
    pub fn transfer_harmonic(
        ensemble: &mut EnsembleEnergy,
        from_id: usize,
        to_id: usize,
        amount: f64,
    ) -> Option<TransferRecord> {
        if from_id == to_id || amount < 0.0 {
            return None;
        }

        let from_idx = ensemble.agents.iter().position(|a| a.agent_id == from_id)?;
        let to_idx = ensemble.agents.iter().position(|a| a.agent_id == to_id)?;

        let actual_amount = amount.min(ensemble.agents[from_idx].harmonic);
        if actual_amount == 0.0 {
            return None;
        }

        ensemble.agents[from_idx].harmonic -= actual_amount;
        ensemble.agents[to_idx].harmonic += actual_amount;

        Some(TransferRecord {
            from_agent: from_id,
            to_agent: to_id,
            kinetic_transferred: 0.0,
            harmonic_transferred: actual_amount,
        })
    }

    /// Transfer both kinetic and harmonic in one atomic operation.
    ///
    /// The transfer is only applied if at least one component would move a
    /// non-zero amount. Each component is clamped to the sender's available
    /// energy, matching the behaviour of [`transfer_kinetic`] and
    /// [`transfer_harmonic`]. If the request is invalid (`from_id == to_id` or
    /// a negative amount) the ensemble is left unchanged and `None` is returned.
    pub fn transfer_both(
        ensemble: &mut EnsembleEnergy,
        from_id: usize,
        to_id: usize,
        kinetic: f64,
        harmonic: f64,
    ) -> Option<TransferRecord> {
        if from_id == to_id || kinetic < 0.0 || harmonic < 0.0 {
            return None;
        }

        let from_idx = ensemble.agents.iter().position(|a| a.agent_id == from_id)?;
        let to_idx = ensemble.agents.iter().position(|a| a.agent_id == to_id)?;

        let actual_kinetic = kinetic.min(ensemble.agents[from_idx].kinetic);
        let actual_harmonic = harmonic.min(ensemble.agents[from_idx].harmonic);

        if actual_kinetic == 0.0 && actual_harmonic == 0.0 {
            return None;
        }

        ensemble.agents[from_idx].kinetic -= actual_kinetic;
        ensemble.agents[to_idx].kinetic += actual_kinetic;
        ensemble.agents[from_idx].harmonic -= actual_harmonic;
        ensemble.agents[to_idx].harmonic += actual_harmonic;

        Some(TransferRecord {
            from_agent: from_id,
            to_agent: to_id,
            kinetic_transferred: actual_kinetic,
            harmonic_transferred: actual_harmonic,
        })
    }

    /// Verify conservation after a transfer.
    pub fn verify_conservation(
        ensemble_before: &EnsembleEnergy,
        ensemble_after: &EnsembleEnergy,
        tolerance: f64,
    ) -> bool {
        let before_total = ensemble_before.total();
        let after_total = ensemble_after.total();
        (before_total - after_total).abs() <= tolerance
    }

    /// Redistribute energy equally across all agents while conserving total.
    pub fn equalize(ensemble: &mut EnsembleEnergy) {
        let n = ensemble.len();
        if n == 0 {
            return;
        }
        let total_kinetic = ensemble.total_kinetic();
        let total_harmonic = ensemble.total_harmonic();
        let per_kinetic = total_kinetic / n as f64;
        let per_harmonic = total_harmonic / n as f64;
        for agent in &mut ensemble.agents {
            agent.kinetic = per_kinetic;
            agent.harmonic = per_harmonic;
        }
    }

    /// Redistribute conserving total energy using a conservation law.
    pub fn conserve_and_redistribute(ensemble: &mut EnsembleEnergy, law: &ConservationLaw) {
        law.enforce_ensemble(ensemble);
        Self::equalize(ensemble);
        law.enforce_ensemble(ensemble);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::RhythmicEnergy;

    #[test]
    fn test_transfer_kinetic() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 8.0, 2.0),
            RhythmicEnergy::new(1, 2.0, 8.0),
        ]);
        let record = EnergyTransfer::transfer_kinetic(&mut ensemble, 0, 1, 3.0).unwrap();
        assert!((record.kinetic_transferred - 3.0).abs() < 1e-10);
        assert!((ensemble.agents[0].kinetic - 5.0).abs() < 1e-10);
        assert!((ensemble.agents[1].kinetic - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_harmonic() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 2.0, 8.0),
            RhythmicEnergy::new(1, 8.0, 2.0),
        ]);
        let record = EnergyTransfer::transfer_harmonic(&mut ensemble, 0, 1, 4.0).unwrap();
        assert!((record.harmonic_transferred - 4.0).abs() < 1e-10);
        assert!((ensemble.agents[0].harmonic - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_preserves_total() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 8.0, 2.0),
            RhythmicEnergy::new(1, 2.0, 8.0),
        ]);
        let before_total = ensemble.total();
        EnergyTransfer::transfer_kinetic(&mut ensemble, 0, 1, 3.0);
        assert!((ensemble.total() - before_total).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_to_self_none() {
        let mut ensemble = EnsembleEnergy::new(vec![RhythmicEnergy::new(0, 5.0, 5.0)]);
        let result = EnergyTransfer::transfer_kinetic(&mut ensemble, 0, 0, 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_transfer_nonexistent_agent() {
        let mut ensemble = EnsembleEnergy::new(vec![RhythmicEnergy::new(0, 5.0, 5.0)]);
        let result = EnergyTransfer::transfer_kinetic(&mut ensemble, 0, 99, 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_equalize() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 8.0, 2.0),
            RhythmicEnergy::new(1, 2.0, 8.0),
        ]);
        EnergyTransfer::equalize(&mut ensemble);
        assert!((ensemble.agents[0].kinetic - 5.0).abs() < 1e-10);
        assert!((ensemble.agents[1].kinetic - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_verify_conservation() {
        let before = EnsembleEnergy::new(vec![RhythmicEnergy::new(0, 5.0, 5.0)]);
        let after = EnsembleEnergy::new(vec![RhythmicEnergy::new(0, 3.0, 7.0)]);
        assert!(EnergyTransfer::verify_conservation(&before, &after, 1e-10));
    }

    #[test]
    fn test_transfer_both() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 8.0, 4.0),
            RhythmicEnergy::new(1, 2.0, 6.0),
        ]);
        let before_total = ensemble.total();
        let record = EnergyTransfer::transfer_both(&mut ensemble, 0, 1, 3.0, 3.0).unwrap();

        assert!((record.kinetic_transferred - 3.0).abs() < 1e-10);
        assert!((record.harmonic_transferred - 3.0).abs() < 1e-10);
        assert!((ensemble.agents[0].kinetic - 5.0).abs() < 1e-10);
        assert!((ensemble.agents[0].harmonic - 1.0).abs() < 1e-10);
        assert!((ensemble.agents[1].kinetic - 5.0).abs() < 1e-10);
        assert!((ensemble.agents[1].harmonic - 9.0).abs() < 1e-10);
        assert!((ensemble.total() - before_total).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_both_kinetic_only_when_harmonic_unavailable() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 0.0),
            RhythmicEnergy::new(1, 2.0, 3.0),
        ]);
        let before_total = ensemble.total();
        let record = EnergyTransfer::transfer_both(&mut ensemble, 0, 1, 2.0, 1.0).unwrap();

        assert!((record.kinetic_transferred - 2.0).abs() < 1e-10);
        assert!((record.harmonic_transferred).abs() < 1e-10);
        assert!((ensemble.agents[0].kinetic - 3.0).abs() < 1e-10);
        assert!((ensemble.agents[1].kinetic - 4.0).abs() < 1e-10);
        assert!((ensemble.total() - before_total).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_both_harmonic_only_when_kinetic_unavailable() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 0.0, 5.0),
            RhythmicEnergy::new(1, 2.0, 3.0),
        ]);
        let before_total = ensemble.total();
        let record = EnergyTransfer::transfer_both(&mut ensemble, 0, 1, 1.0, 2.0).unwrap();

        assert!((record.kinetic_transferred).abs() < 1e-10);
        assert!((record.harmonic_transferred - 2.0).abs() < 1e-10);
        assert!((ensemble.agents[0].harmonic - 3.0).abs() < 1e-10);
        assert!((ensemble.agents[1].harmonic - 5.0).abs() < 1e-10);
        assert!((ensemble.total() - before_total).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_both_negative_amount_none() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
            RhythmicEnergy::new(1, 2.0, 3.0),
        ]);
        let before = ensemble.clone();
        let result = EnergyTransfer::transfer_both(&mut ensemble, 0, 1, -1.0, 1.0);

        assert!(result.is_none());
        assert_eq!(ensemble.agents, before.agents);
    }

    #[test]
    fn test_transfer_both_clamps_to_available() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 1.0, 2.0),
            RhythmicEnergy::new(1, 0.0, 0.0),
        ]);
        let record = EnergyTransfer::transfer_both(&mut ensemble, 0, 1, 100.0, 100.0).unwrap();

        assert!((record.kinetic_transferred - 1.0).abs() < 1e-10);
        assert!((record.harmonic_transferred - 2.0).abs() < 1e-10);
        assert!((ensemble.agents[0].kinetic).abs() < 1e-10);
        assert!((ensemble.agents[0].harmonic).abs() < 1e-10);
        assert!((ensemble.agents[1].kinetic - 1.0).abs() < 1e-10);
        assert!((ensemble.agents[1].harmonic - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_transfer_both_to_self_none() {
        let mut ensemble = EnsembleEnergy::new(vec![RhythmicEnergy::new(0, 5.0, 5.0)]);
        let result = EnergyTransfer::transfer_both(&mut ensemble, 0, 0, 1.0, 1.0);
        assert!(result.is_none());
    }
}
