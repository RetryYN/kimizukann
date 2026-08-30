use kimizukann_sim_core::SimCore;
use kimizukann_sim_types::{CellState, Fixed, TickPhase, FIXED_SCALE};

const C: Fixed = 50_000;
const MMAX: Fixed = 200_000_000_000_000;

fn blank() -> CellState {
    CellState {
        nutrient: 0,
        biomass: [0; 8],
        carcass: 0,
        waste: 0,
        energy: [FIXED_SCALE / 2; 8],
        occupancy_peak: 0,
    }
}

fn pair(nutrient: Fixed) -> SimCore {
    let mut a = blank();
    a.nutrient = nutrient;
    SimCore::try_grid(2, 1, 7, vec![a, blank()], vec![]).unwrap()
}

#[test]
fn ut_d2_02_corner_edge_and_no_wrap() {
    assert_eq!(
        SimCore::neighbor_indices(64, 64, 0)
            .iter()
            .flatten()
            .count(),
        2
    );
    assert_eq!(
        SimCore::neighbor_indices(64, 64, 32)
            .iter()
            .flatten()
            .count(),
        3
    );
    let mut cells = vec![blank(); 3];
    cells[0].nutrient = FIXED_SCALE;
    let mut s = SimCore::try_grid(3, 1, 1, cells, vec![]).unwrap();
    s.apply_phase(TickPhase::Diffuse).unwrap();
    assert_eq!(s.total_mass(), FIXED_SCALE);
    assert_eq!(s.state.grid.cells[2].nutrient, 0);
    assert_eq!(
        s.state.grid.cells[0].nutrient + s.state.grid.cells[1].nutrient,
        FIXED_SCALE
    );
}

#[test]
fn ut_d2_03_uniform_hash_stable() {
    let cells = vec![
        CellState {
            nutrient: FIXED_SCALE,
            ..blank()
        };
        64 * 64
    ];
    let mut s = SimCore::try_grid(64, 64, 9, cells, vec![]).unwrap();
    let h = s.state_hash();
    for _ in 0..2_000 {
        s.apply_phase(TickPhase::Diffuse).unwrap();
    }
    assert_eq!(s.state_hash(), h);
    assert_eq!(s.total_mass(), FIXED_SCALE * 64 * 64);
}

#[test]
fn ut_d2_04_zero_coeff_unchanged() {
    let mut a = blank();
    a.nutrient = FIXED_SCALE;
    a.waste = 7;
    let mut s = SimCore::try_grid(2, 1, 2, vec![a, blank()], vec![]).unwrap();
    s.diffusion_coefficients = [0, C, 0, 0];
    s.apply_phase(TickPhase::Diffuse).unwrap();
    assert_eq!(s.state.grid.cells[0].nutrient, FIXED_SCALE);
    assert_eq!(s.state.grid.cells[0].waste, 7);
}

#[test]
fn ut_d2_06_mmax_i128() {
    assert_eq!(
        SimCore::outflow_amount(MMAX, C).unwrap(),
        10_000_000_000_000
    );
    let mut s = pair(MMAX);
    s.apply_phase(TickPhase::Diffuse).unwrap();
    assert_eq!(s.total_mass(), MMAX);
}

#[test]
fn ut_d2_08_static_region_id() {
    assert_eq!(SimCore::static_region_id(64, 64, 0), 0);
    assert_eq!(SimCore::static_region_id(64, 64, 16), 1);
    assert_eq!(SimCore::static_region_id(64, 64, 16 * 64), 4);
    assert_eq!(SimCore::static_region_id(64, 64, 64 * 64 - 1), 15);
}
