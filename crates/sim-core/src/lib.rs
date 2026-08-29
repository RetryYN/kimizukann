//! D0 headless core skeleton. Implementation begins in D1.

use kimizukann_sim_types::{InvariantReport, Seed, WorldState};

#[derive(Debug)]
pub struct SimCore { pub state: WorldState, pub seed: Seed }

impl SimCore {
    pub fn step(&mut self, _ticks: u32) -> Result<(), String> {
        // TODO(D1): implement fixed-order tick phases and ledgers.
        Ok(())
    }

    pub fn invariant_report(&self) -> InvariantReport {
        // TODO(D1): calculate mass/energy conservation and non-negative checks.
        InvariantReport { mass_ok: false, energy_ok: false, non_negative: false, message: "D0 skeleton".into() }
    }
}
