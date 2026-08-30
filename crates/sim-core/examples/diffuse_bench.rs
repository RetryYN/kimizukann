//! Load path for REQ-NFR-01 (diffuse-only 2,000 tick on 64×64).
//! `cargo run -p kimizukann-sim-core --example diffuse_bench --release`
//! Wall clock is forbidden (REQ-CON-05 / REQ-DET-04a / BD-07 §4.2).
//! This example only exercises the 2,000-tick path; timing is a later CI job.

use kimizukann_sim_core::SimCore;
use kimizukann_sim_types::{CellState, TickPhase, FIXED_SCALE};

fn main() {
    let mut cells = vec![
        CellState {
            nutrient: 0,
            biomass: [0; 8],
            carcass: 0,
            waste: 0,
            energy: [FIXED_SCALE / 2; 8],
            occupancy_peak: 0,
        };
        64 * 64
    ];
    cells[0].nutrient = FIXED_SCALE;
    let mut s = SimCore::try_grid(64, 64, 1, cells, vec![]).unwrap();
    s.apply_phase(TickPhase::Diffuse).unwrap();
    for _ in 0..2_000 {
        s.apply_phase(TickPhase::Diffuse).unwrap();
    }
    println!("diffuse_2000_64x64 ticks=2000");
}
