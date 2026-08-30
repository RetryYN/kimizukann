use kimizukann_sim_core::{epsilon, judge, ranking, share_ge, step_streak, TermState};
use kimizukann_sim_types::{
    CellState, Fixed, GridState, LineageParams, MechanismTags, TerminationLabel, Thresholds,
    TraitVector, WorldState, FIXED_SCALE,
};

fn lin(id: u8) -> LineageParams {
    LineageParams {
        id,
        traits: TraitVector {
            movement: FIXED_SCALE,
            intake: FIXED_SCALE,
            conversion: FIXED_SCALE,
            maintenance_cost: FIXED_SCALE,
            reproduction: FIXED_SCALE,
        },
        tags: MechanismTags::default(),
        mortality_threshold: 1,
        waste_emission: 1,
    }
}

fn world(tick: u32, m: [Fixed; 4]) -> WorldState {
    let mut cell = CellState {
        nutrient: 0,
        biomass: [0; 8],
        carcass: 0,
        waste: 0,
        energy: [0; 8],
        occupancy_peak: 0,
    };
    for (i, v) in m.into_iter().enumerate() {
        cell.biomass[i] = v;
    }
    WorldState {
        tick,
        grid: GridState {
            width: 1,
            height: 1,
            cells: vec![cell],
        },
        lineages: (0..4).map(lin).collect(),
    }
}

fn th() -> Thresholds {
    Thresholds {
        base_intake: 0,
        base_maintenance: 0,
        epsilon: 400,
        fixed_share: 700_000,
        fixed_ticks: 200,
        coexist_share: 150_000,
        max_ticks: 2_000,
        waste_toxic_threshold: 0,
        toxin_maintenance_multiplier: 0,
        occupancy_threshold: 0,
        vacant_nutrient_threshold: 0,
    }
}

fn term(streak: u32, initial: Fixed, rank: Vec<u8>) -> TermState {
    TermState {
        fixed_streak: streak,
        streak_id: Some(0),
        tick0_ranking: rank,
        initial_total_biomass: initial,
    }
}

#[test]
fn ut_d4_02_epsilon_from_initial() {
    // REQ-END-02
    assert_eq!(epsilon(4_000_000).unwrap(), 400);
}

#[test]
fn ut_d4_03_extinct_boundary() {
    // REQ-END-02
    let t = th();
    let low = world(1, [399, 0, 0, 0]);
    let edge = world(1, [400, 0, 0, 0]);
    let st = term(0, 4_000_000, vec![0, 1, 2, 3]);
    assert_eq!(judge(&low, &st, &t, false).unwrap(), None);
    assert_eq!(
        judge(&low, &st, &t, true).unwrap(),
        Some(TerminationLabel::Extinct)
    );
    assert_eq!(judge(&edge, &st, &t, true).unwrap(), None);
}

#[test]
fn ut_d4_04_fixed_streak_200() {
    // REQ-END-03
    let t = th();
    let w = world(10, [700_000, 300_000, 0, 0]);
    assert!(share_ge(700_000, 1_000_000, 700_000));
    assert!(!share_ge(699_999, 1_000_000, 700_000));
    let a = term(199, 4_000_000, vec![0, 1, 2, 3]);
    let b = term(200, 4_000_000, vec![0, 1, 2, 3]);
    assert_eq!(judge(&w, &a, &t, true).unwrap(), None);
    assert_eq!(
        judge(&w, &b, &t, true).unwrap(),
        Some(TerminationLabel::Fixed)
    );
    let miss = world(10, [699_999, 300_001, 0, 0]);
    assert_eq!(judge(&miss, &b, &t, true).unwrap(), None);
}

#[test]
fn ut_d4_05_streak_reset() {
    // REQ-END-03
    let hold = world(5, [700_000, 300_000, 0, 0]);
    let steal = world(6, [300_000, 700_000, 0, 0]);
    let broken = world(6, [500_000, 500_000, 0, 0]);
    let mut st = term(50, 4_000_000, vec![0, 1, 2, 3]);
    step_streak(&mut st, &hold).unwrap();
    assert_eq!((st.fixed_streak, st.streak_id), (51, Some(0)));
    step_streak(&mut st, &steal).unwrap();
    assert_eq!((st.fixed_streak, st.streak_id), (1, Some(1)));
    step_streak(&mut st, &broken).unwrap();
    assert_eq!((st.fixed_streak, st.streak_id), (0, None));
}

#[test]
fn ut_d4_06_coexist_at_limit() {
    // REQ-END-04a
    let t = th();
    let w = world(2_000, [150_000, 150_000, 700_000, 0]);
    let st = term(0, 4_000_000, vec![0, 1, 2, 3]);
    assert_eq!(
        judge(&w, &st, &t, true).unwrap(),
        Some(TerminationLabel::Coexist)
    );
}

#[test]
fn ut_d4_07_tick0_tie_id_order() {
    // REQ-END-04b
    let w = world(0, [1_000_000, 1_000_000, 1_000_000, 1_000_000]);
    assert_eq!(ranking(&w).unwrap(), vec![0, 1, 2, 3]);
}

#[test]
fn ut_d4_08_reversal_vs_timelimit() {
    // REQ-END-04a REQ-END-04c
    let t = th();
    let w = world(2_000, [900_000, 40_000, 40_000, 20_000]);
    let rev = term(0, 4_000_000, vec![2, 1, 0, 3]);
    let lim = term(0, 4_000_000, vec![1, 0, 2, 3]);
    assert_eq!(
        judge(&w, &rev, &t, true).unwrap(),
        Some(TerminationLabel::Reversal)
    );
    assert_eq!(
        judge(&w, &lim, &t, true).unwrap(),
        Some(TerminationLabel::TimeLimit)
    );
}

#[test]
fn ut_d4_09_extinct_beats_fixed() {
    // REQ-END-04c
    let t = th();
    let w = world(10, [300, 50, 0, 0]);
    let st = term(200, 4_000_000, vec![0, 1, 2, 3]);
    assert_eq!(
        judge(&w, &st, &t, true).unwrap(),
        Some(TerminationLabel::Extinct)
    );
}
