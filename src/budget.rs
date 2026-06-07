//! Energy budget allocation for ensemble agents.

use crate::energy::{EnsembleEnergy, RhythmicEnergy};

/// Budget allocation strategy.
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStrategy {
    /// Equal distribution among agents.
    Equal,
    /// Proportional to current energy.
    Proportional,
    /// Weighted distribution.
    Weighted(Vec<f64>),
}

/// Energy budget for the ensemble.
#[derive(Debug, Clone)]
pub struct EnergyBudget {
    /// Total energy budget C.
    pub total_budget: f64,
    /// Strategy for allocation.
    pub strategy: AllocationStrategy,
}

impl EnergyBudget {
    /// Create a new energy budget.
    pub fn new(total_budget: f64, strategy: AllocationStrategy) -> Self {
        Self { total_budget, strategy }
    }

    /// Allocate energy equally among n agents.
    pub fn allocate_equal(&self, n_agents: usize) -> EnsembleEnergy {
        if n_agents == 0 {
            return EnsembleEnergy::new(vec![]);
        }
        let per_agent = self.total_budget / n_agents as f64;
        let agents = (0..n_agents)
            .map(|id| RhythmicEnergy::new(id, per_agent / 2.0, per_agent / 2.0))
            .collect();
        EnsembleEnergy::new(agents)
    }

    /// Allocate energy proportionally to existing agent energies.
    pub fn allocate_proportional(&self, current: &EnsembleEnergy) -> EnsembleEnergy {
        let current_total = current.total();
        if current_total == 0.0 {
            return self.allocate_equal(current.len());
        }
        let agents = current.agents.iter().map(|a| {
            let fraction = a.total() / current_total;
            let new_total = fraction * self.total_budget;
            let ratio = a.kinetic_ratio();
            RhythmicEnergy::new(
                a.agent_id,
                new_total * ratio,
                new_total * (1.0 - ratio),
            )
        }).collect();
        EnsembleEnergy::new(agents)
    }

    /// Allocate energy with explicit weights.
    pub fn allocate_weighted(&self, weights: &[f64]) -> EnsembleEnergy {
        let weight_sum: f64 = weights.iter().sum();
        if weight_sum == 0.0 {
            return self.allocate_equal(weights.len());
        }
        let agents = weights.iter().enumerate().map(|(id, w)| {
            let fraction = w / weight_sum;
            let new_total = fraction * self.total_budget;
            RhythmicEnergy::new(id, new_total / 2.0, new_total / 2.0)
        }).collect();
        EnsembleEnergy::new(agents)
    }

    /// Check if an allocation exceeds the budget.
    pub fn is_within_budget(&self, ensemble: &EnsembleEnergy) -> bool {
        ensemble.total() <= self.total_budget + 1e-10
    }

    /// Reserve a portion of the budget, returning the remainder.
    pub fn reserve(&self, amount: f64) -> EnergyBudget {
        EnergyBudget::new(
            (self.total_budget - amount).max(0.0),
            self.strategy.clone(),
        )
    }

    /// Distribute budget according to the configured strategy.
    pub fn distribute(&self, n_agents: usize, current: Option<&EnsembleEnergy>) -> EnsembleEnergy {
        match &self.strategy {
            AllocationStrategy::Equal => self.allocate_equal(n_agents),
            AllocationStrategy::Proportional => {
                if let Some(ens) = current {
                    self.allocate_proportional(ens)
                } else {
                    self.allocate_equal(n_agents)
                }
            }
            AllocationStrategy::Weighted(w) => self.allocate_weighted(w),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_allocation() {
        let budget = EnergyBudget::new(10.0, AllocationStrategy::Equal);
        let ensemble = budget.allocate_equal(2);
        assert!((ensemble.total() - 10.0).abs() < 1e-10);
        assert!((ensemble.agents[0].total() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_proportional_allocation() {
        let budget = EnergyBudget::new(20.0, AllocationStrategy::Proportional);
        let current = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 3.0, 7.0),
            RhythmicEnergy::new(1, 7.0, 3.0),
        ]);
        let allocated = budget.allocate_proportional(&current);
        assert!((allocated.total() - 20.0).abs() < 1e-10);
        // Agent 1 had more total energy (10 vs 10 here, equal)
    }

    #[test]
    fn test_weighted_allocation() {
        let budget = EnergyBudget::new(10.0, AllocationStrategy::Weighted(vec![1.0, 3.0]));
        let ensemble = budget.allocate_weighted(&[1.0, 3.0]);
        assert!((ensemble.total() - 10.0).abs() < 1e-10);
        assert!((ensemble.agents[1].total() - 7.5).abs() < 1e-10);
    }

    #[test]
    fn test_within_budget() {
        let budget = EnergyBudget::new(10.0, AllocationStrategy::Equal);
        let ensemble = EnsembleEnergy::new(vec![
            RhythmicEnergy::new(0, 3.0, 3.0),
        ]);
        assert!(budget.is_within_budget(&ensemble));
    }

    #[test]
    fn test_reserve() {
        let budget = EnergyBudget::new(10.0, AllocationStrategy::Equal);
        let remaining = budget.reserve(3.0);
        assert!((remaining.total_budget - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_allocation() {
        let budget = EnergyBudget::new(10.0, AllocationStrategy::Equal);
        let ensemble = budget.allocate_equal(0);
        assert!(ensemble.is_empty());
    }
}
