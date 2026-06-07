//! Energy transfer between agents while maintaining conservation.

use crate::energy::EnsembleEnergy;
use crate::conservation::ConservationLaw;

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

    /// Transfer both kinetic and harmonic in one operation.
    pub fn transfer_both(
        ensemble: &mut EnsembleEnergy,
        from_id: usize,
        to_id: usize,
        kinetic: f64,
        harmonic: f64,
    ) -> Option<TransferRecord> {
        let kt = Self::transfer_kinetic(ensemble, from_id, to_id, kinetic)?;
        // Undo and redo as combined
        // Actually, just do harmonic separately on the modified ensemble
        let ht = Self::transfer_harmonic(ensemble, from_id, to_id, harmonic);

        Some(TransferRecord {
            from_agent: from_id,
            to_agent: to_id,
            kinetic_transferred: kt.kinetic_transferred,
            harmonic_transferred: ht.map(|h| h.harmonic_transferred).unwrap_or(0.0),
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
    pub fn conserve_and_redistribute(
        ensemble: &mut EnsembleEnergy,
        law: &ConservationLaw,
    ) {
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
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
        let result = EnergyTransfer::transfer_kinetic(&mut ensemble, 0, 0, 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_transfer_nonexistent_agent() {
        let mut ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
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
        let before = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 5.0, 5.0),
        ]);
        let after = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 3.0, 7.0),
        ]);
        assert!(EnergyTransfer::verify_conservation(&before, &after, 1e-10));
    }
}
