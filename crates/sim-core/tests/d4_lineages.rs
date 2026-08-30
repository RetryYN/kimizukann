use kimizukann_sim_core::{preset_v1, PlaceMode, SimCore, Xoshiro256StarStar};
use kimizukann_sim_types::{Fixed, LineageParams, FIXED_SCALE};

fn over_under(l: &LineageParams) -> bool {
    let t = [
        l.traits.movement,
        l.traits.intake,
        l.traits.conversion,
        l.traits.maintenance_cost,
        l.traits.reproduction,
    ];
    t.iter().any(|&x| x > FIXED_SCALE) && t.iter().any(|&x| x < FIXED_SCALE)
}

fn words_after(seed: u64, xor: u64, n: usize) -> [u64; 4] {
    let mut r = Xoshiro256StarStar::from_seed(seed ^ xor);
    for _ in 0..n {
        r.next_u64();
    }
    r.words()
}

fn occupied(s: &SimCore) -> Vec<(usize, u8, Fixed)> {
    let mut out = Vec::new();
    for (i, c) in s.state.grid.cells.iter().enumerate() {
        for id in 0..8u8 {
            if c.biomass[id as usize] > 0 {
                out.push((i, id, c.biomass[id as usize]));
            }
        }
    }
    out
}

#[test]
fn ut_d4_01_preset_no_simple_dominance() {
    // REQ-GEN-04
    let p = preset_v1();
    assert!(over_under(&p[0]) && over_under(&p[1]) && over_under(&p[2]));
    assert!(
        p[3].traits.movement <= FIXED_SCALE
            && p[3].traits.intake <= FIXED_SCALE
            && p[3].traits.conversion <= FIXED_SCALE
            && p[3].traits.maintenance_cost <= FIXED_SCALE
            && p[3].traits.reproduction <= FIXED_SCALE
    );
    assert_eq!(p[2].mortality_threshold, 10_000);
    assert_eq!(p[2].waste_emission, 2_000);
    assert!(p[0].tags.toxin_sensitive && p[3].tags.use_carcass && p[3].tags.use_waste);
    assert!(p.iter().all(|l| !l.tags.density_bonus));
}

#[test]
fn ut_d4_10_variation_upper_plus_one() {
    // REQ-GEN-08
    let p = preset_v1().to_vec();
    let modes = vec![PlaceMode::Default; 4];
    let mass = [1_000_000; 4];
    assert!(SimCore::try_d4(1, p.clone(), &modes, &mass, 1_000_001).is_err());
    assert!(SimCore::try_d4(1, p, &modes, &mass, FIXED_SCALE).is_ok());
}

#[test]
fn ut_d4_11_random_same_seed() {
    // REQ-SCOPE-03
    let p = preset_v1()[..2].to_vec();
    let modes = vec![PlaceMode::Random { k: 3 }; 2];
    let mass = [1_000_000, 1_000_000];
    let a = SimCore::try_d4(99, p.clone(), &modes, &mass, 0).unwrap();
    let b = SimCore::try_d4(99, p.clone(), &modes, &mass, 0).unwrap();
    assert_eq!(occupied(&a), occupied(&b));
    assert_eq!(a.rng[3].words(), words_after(99, 3, 6));
    assert_eq!(a.rng[2].words(), words_after(99, 2, 2));
    let def = vec![
        PlaceMode::Default,
        PlaceMode::Default,
        PlaceMode::Default,
        PlaceMode::Default,
    ];
    let d = SimCore::try_d4(7, preset_v1().to_vec(), &def, &[1_000_000; 4], 0).unwrap();
    assert_eq!(d.rng[3].words(), words_after(7, 3, 0));
    for (id, (x, y)) in [(16u16, 16u16), (48, 16), (16, 48), (48, 48)]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            d.state.grid.cells[y as usize * 64 + x as usize].biomass[id],
            1_000_000
        );
    }
    let varied =
        SimCore::try_d4(3, preset_v1().to_vec(), &def, &[1_000_000; 4], FIXED_SCALE).unwrap();
    assert_eq!(varied.state.lineages[0].traits.conversion, 950_000);
}

#[test]
fn ut_d4_12_random_full_grid() {
    let p = preset_v1()[..1].to_vec();
    let s = SimCore::try_d4(5, p, &[PlaceMode::Random { k: 4096 }], &[1_000_000], 0).unwrap();
    let occ = occupied(&s);
    assert_eq!(occ.len(), 4096);
    assert!(occ.iter().all(|&(_, id, m)| id == 0 && m == 1_000_000));
    assert_eq!(s.rng[3].words(), words_after(5, 3, 4096));
}
