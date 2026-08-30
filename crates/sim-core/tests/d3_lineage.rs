//! Failing UT/PT for DD-D3 §9. Implementation follows in a later commit.

use kimizukann_sim_core::SimCore;
use kimizukann_sim_types::{
    CellState, Fixed, LineageParams, MechanismTags, ReasonCode, TickPhase, TraitVector, FIXED_SCALE,
};

const BASE_INTAKE: Fixed = 100_000;
const BASE_MAINT: Fixed = 10_000;

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

fn lin(id: u8, tags: MechanismTags) -> LineageParams {
    LineageParams {
        id,
        traits: TraitVector {
            movement: 0,
            intake: FIXED_SCALE,
            conversion: FIXED_SCALE,
            maintenance_cost: FIXED_SCALE,
            reproduction: FIXED_SCALE,
        },
        tags,
        mortality_threshold: 5_000,
        waste_emission: 1_000,
    }
}

fn nutrient_tag() -> MechanismTags {
    MechanismTags {
        use_nutrient: true,
        ..MechanismTags::default()
    }
}

fn core(cell: CellState, lineages: Vec<LineageParams>) -> SimCore {
    SimCore::try_grid(1, 1, 7, vec![cell], lineages).unwrap()
}

#[test]
fn ut_d3_01_no_split_intake() {
    let mut cell = blank();
    cell.nutrient = 150_000;
    cell.biomass = [FIXED_SCALE, FIXED_SCALE, 0, 0, 0, 0, 0, 0];
    let mut s = core(cell, vec![lin(0, nutrient_tag()), lin(1, nutrient_tag())]);
    s.apply_phase(TickPhase::Intake).unwrap();
    let c = &s.state.grid.cells[0];
    assert_eq!(c.nutrient, 0);
    assert!(c.biomass[0] > c.biomass[1]);
}

#[test]
fn ut_d3_02_intake_cap() {
    let mut cell = blank();
    cell.nutrient = FIXED_SCALE;
    cell.biomass[0] = FIXED_SCALE;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    let before = s.state.grid.cells[0].nutrient;
    s.apply_phase(TickPhase::Intake).unwrap();
    assert_eq!(before - s.state.grid.cells[0].nutrient, BASE_INTAKE);
}

#[test]
fn ut_d3_03_carcass_not_nutrient() {
    let mut cell = blank();
    cell.nutrient = FIXED_SCALE;
    cell.carcass = FIXED_SCALE;
    cell.biomass[0] = FIXED_SCALE;
    let tags = MechanismTags {
        use_carcass: true,
        ..MechanismTags::default()
    };
    let mut s = core(cell, vec![lin(0, tags)]);
    s.apply_phase(TickPhase::Intake).unwrap();
    let c = &s.state.grid.cells[0];
    assert_eq!(c.nutrient, FIXED_SCALE);
    assert!(c.carcass < FIXED_SCALE);
}

#[test]
fn ut_d3_04_energy_heat() {
    let mut cell = blank();
    cell.nutrient = FIXED_SCALE;
    cell.biomass[0] = FIXED_SCALE;
    cell.energy[0] = FIXED_SCALE - 10;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.apply_phase(TickPhase::Intake).unwrap();
    assert_eq!(s.state.grid.cells[0].energy[0], FIXED_SCALE);
    let heat: Fixed = s
        .energy_records()
        .iter()
        .filter(|r| r.reason == ReasonCode::Intake && r.from_pool != r.to_pool)
        .map(|r| r.amount)
        .sum();
    assert!(heat > 0);
}

#[test]
fn ut_d3_05_toxin_multiplier() {
    let mut cell = blank();
    cell.biomass[0] = FIXED_SCALE;
    cell.waste = 200_000;
    cell.energy[0] = FIXED_SCALE;
    let sensitive = MechanismTags {
        use_nutrient: true,
        toxin_sensitive: true,
        ..MechanismTags::default()
    };
    let hardy = MechanismTags {
        use_nutrient: true,
        ..MechanismTags::default()
    };
    let mut a = core(cell.clone(), vec![lin(0, sensitive)]);
    let mut b = core(cell, vec![lin(0, hardy)]);
    a.apply_phase(TickPhase::Maintenance).unwrap();
    b.apply_phase(TickPhase::Maintenance).unwrap();
    let lost_a = FIXED_SCALE - a.state.grid.cells[0].energy[0];
    let lost_b = FIXED_SCALE - b.state.grid.cells[0].energy[0];
    assert_eq!(lost_b, BASE_MAINT);
    assert_eq!(lost_a, fixed_mul_14(BASE_MAINT));
}

fn fixed_mul_14(cost: Fixed) -> Fixed {
    ((cost as i128) * 1_400_000 / 1_000_000) as Fixed
}

#[test]
fn ut_d3_06_starving_on_shortfall() {
    let mut cell = blank();
    cell.biomass[0] = FIXED_SCALE;
    cell.energy[0] = 3_000;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.apply_phase(TickPhase::Maintenance).unwrap();
    assert_eq!(s.state.grid.cells[0].energy[0], 0);
    assert_eq!(s.life_of(0, 0), 2);
    assert_eq!(s.last_deficit(0, 0), BASE_MAINT - 3_000);
}

#[test]
fn ut_d3_07_starvation_partial() {
    let mut cell = blank();
    cell.biomass[0] = 80_000;
    cell.energy[0] = 0;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.set_life(0, 0, 2);
    s.set_deficit(0, 0, 10_000);
    s.apply_phase(TickPhase::StarvationAndDeath).unwrap();
    let c = &s.state.grid.cells[0];
    assert_eq!(c.biomass[0], 70_000);
    assert_eq!(c.carcass, 10_000);
    assert_eq!(s.life_of(0, 0), 1);
}

#[test]
fn ut_d3_08_death_to_absent() {
    let mut cell = blank();
    cell.biomass[0] = 3_000;
    cell.energy[0] = 0;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.set_life(0, 0, 2);
    s.set_deficit(0, 0, 10_000);
    s.apply_phase(TickPhase::StarvationAndDeath).unwrap();
    let c = &s.state.grid.cells[0];
    assert_eq!(c.biomass[0], 0);
    assert_eq!(c.carcass, 3_000);
    assert_eq!(s.life_of(0, 0), 0);
}

#[test]
fn ut_d3_09_no_repro_no_rng() {
    let mut cell = blank();
    cell.biomass[0] = FIXED_SCALE;
    cell.nutrient = FIXED_SCALE;
    cell.energy[0] = 15_000;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    let before = s.rng[1].words();
    s.apply_phase(TickPhase::Reproduction).unwrap();
    assert_eq!(s.rng[1].words(), before);
    assert_eq!(s.state.grid.cells[0].biomass[0], FIXED_SCALE);
}

#[test]
fn ut_d3_10_repro_consumes_one() {
    let mut cell = blank();
    cell.biomass[0] = FIXED_SCALE;
    cell.nutrient = FIXED_SCALE;
    cell.energy[0] = 100_000;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    let before = s.rng[1].words();
    s.apply_phase(TickPhase::Reproduction).unwrap();
    assert_ne!(s.rng[1].words(), before);
    let mut replay = s.rng[1];
    // one word must have been taken; a second next_u64 would differ again
    let _ = replay.next_u64();
    assert_ne!(replay.words(), s.rng[1].words());
}

#[test]
fn ut_d3_11_repro_mass() {
    let mut cell = blank();
    cell.biomass[0] = FIXED_SCALE;
    cell.nutrient = FIXED_SCALE;
    cell.energy[0] = 100_000;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    let n0 = s.state.grid.cells[0].nutrient;
    let b0 = s.state.grid.cells[0].biomass[0];
    let e0 = s.state.grid.cells[0].energy[0];
    s.apply_phase(TickPhase::Reproduction).unwrap();
    let c = &s.state.grid.cells[0];
    assert_eq!(n0 - c.nutrient, c.biomass[0] - b0);
    assert_eq!(e0 - c.energy[0], c.biomass[0] - b0);
}

#[test]
fn ut_d3_12_emission() {
    let mut cell = blank();
    cell.biomass[0] = 50_000;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.apply_phase(TickPhase::Emission).unwrap();
    let c = &s.state.grid.cells[0];
    assert_eq!(c.biomass[0], 49_000);
    assert_eq!(c.waste, 1_000);
}

#[test]
fn ut_d3_13_ledger_sorted() {
    let mut cell = blank();
    cell.nutrient = FIXED_SCALE;
    cell.biomass[0] = FIXED_SCALE;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.apply_phase(TickPhase::Intake).unwrap();
    s.fold_lineage_records();
    let recs = s.ledger_records();
    assert!(!recs.is_empty());
    let keys: Vec<_> = recs
        .iter()
        .map(|r| (r.tick, r.region_id, r.lineage, r.reason as u8, r.from_pool as u8, r.to_pool as u8))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

#[test]
fn ut_d3_14_prng_table() {
    let mut cell = blank();
    cell.nutrient = FIXED_SCALE;
    cell.biomass[0] = FIXED_SCALE;
    cell.energy[0] = 100_000;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    let r0 = s.rng[1].words();
    s.step(1).unwrap();
    assert_ne!(s.rng[1].words(), r0);
    assert_eq!(s.rng[0].words(), SimCore::one_cell(7, 0, 0, vec![]).rng[0].words());
}

#[test]
fn pt_d3_01_states_and_mass() {
    let mut cell = blank();
    cell.nutrient = 10 * FIXED_SCALE;
    for i in 0..8 {
        cell.biomass[i] = 20_000;
        cell.energy[i] = FIXED_SCALE / 2;
    }
    let lins: Vec<_> = (0..8).map(|i| lin(i, nutrient_tag())).collect();
    let mut s = core(cell, lins);
    let mass = s.total_mass();
    s.step(8).unwrap();
    assert_eq!(s.total_mass(), mass);
    for id in 0..8 {
        let life = s.life_of(0, id);
        assert!(life == 0 || life == 1 || life == 2);
    }
}

#[test]
fn pt_d3_02_invariants() {
    let mut cell = blank();
    cell.nutrient = 8 * FIXED_SCALE;
    cell.biomass[0] = FIXED_SCALE;
    let mut s = core(cell, vec![lin(0, nutrient_tag())]);
    s.step(64).unwrap();
    let r = s.invariant_report();
    assert!(r.mass_ok && r.non_negative);
    for e in s.state.grid.cells[0].energy {
        assert!((0..=FIXED_SCALE).contains(&e));
    }
}
