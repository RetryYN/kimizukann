use kimizukann_sim_types::{
    CellState, Fixed, LineageParams, MechanismTags, TraitVector, FIXED_SCALE,
};

use super::SimCore;

const W: u16 = 64;
const N: usize = 4096;
const Q: [(u16, u16); 4] = [(16, 16), (48, 16), (16, 48), (48, 48)];
const ALLELE: Fixed = 50_000;
const MASS_MAX: Fixed = 200_000_000_000_000;

#[derive(Clone, Debug)]
pub enum PlaceMode {
    Default,
    Explicit(Vec<(u16, u16)>),
    Random { k: u16 },
}

fn tv(a: [Fixed; 5]) -> TraitVector {
    TraitVector {
        movement: a[0],
        intake: a[1],
        conversion: a[2],
        maintenance_cost: a[3],
        reproduction: a[4],
    }
}

fn tg(n: bool, c: bool, w: bool, t: bool) -> MechanismTags {
    MechanismTags {
        use_nutrient: n,
        use_carcass: c,
        use_waste: w,
        toxin_sensitive: t,
        density_bonus: false,
    }
}

fn lp(id: u8, a: [Fixed; 5], g: MechanismTags, m: Fixed, e: Fixed) -> LineageParams {
    LineageParams {
        id,
        traits: tv(a),
        tags: g,
        mortality_threshold: m,
        waste_emission: e,
    }
}

#[rustfmt::skip]
pub fn preset_v1() -> [LineageParams; 4] {
    [
        lp(0, [700_000, 1_050_000, 950_000, 1_000_000, 850_000], tg(true, false, false, true), 5_000, 1_000),
        lp(1, [1_200_000, 800_000, 900_000, 1_100_000, 850_000], tg(true, false, false, false), 5_000, 1_000),
        lp(2, [1_000_000, 1_150_000, 850_000, 1_300_000, 1_600_000], tg(true, false, false, false), 10_000, 2_000),
        lp(3, [800_000, 450_000, 950_000, 900_000, 650_000], tg(true, true, true, false), 5_000, 1_000),
    ]
}

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

impl SimCore {
    pub fn try_d4(
        seed: u64,
        lineages: Vec<LineageParams>,
        modes: &[PlaceMode],
        initial_biomass: &[Fixed],
        variation: Fixed,
    ) -> Result<Self, String> {
        if !(0..=FIXED_SCALE).contains(&variation) {
            return Err("variation out of range".into());
        }
        if lineages.is_empty()
            || modes.len() != lineages.len()
            || initial_biomass.len() != lineages.len()
        {
            return Err("lineage/mode/biomass length".into());
        }
        if initial_biomass.iter().any(|&x| x <= 0 || x > MASS_MAX) {
            return Err("initial_biomass".into());
        }
        let mut pack: Vec<_> = lineages
            .into_iter()
            .zip(modes.iter().cloned())
            .zip(initial_biomass.iter().copied())
            .map(|((l, m), b)| (l, m, b))
            .collect();
        pack.sort_by_key(|(l, _, _)| l.id);
        if pack.windows(2).any(|w| w[0].0.id == w[1].0.id) {
            return Err("duplicate lineage id".into());
        }
        let only = pack.iter().map(|(l, _, _)| *l).collect();
        let mut core = Self::try_grid(W, W, seed, vec![blank(); N], only)?;
        for lineage in &mut core.state.lineages {
            let word = core.rng[2].next_u64();
            if variation == 0 {
                continue;
            }
            let t = &mut lineage.traits;
            let slots = [
                &mut t.movement,
                &mut t.intake,
                &mut t.conversion,
                &mut t.maintenance_cost,
                &mut t.reproduction,
            ];
            for (i, slot) in slots.into_iter().enumerate() {
                if i == 2 {
                    continue;
                }
                let bits = ((word >> (i * 12)) & 0xFFF) as i64;
                *slot += (bits % 3 - 1) * ALLELE;
            }
        }
        let mut taken = [false; N];
        for (i, (_, mode, mass)) in pack.iter().enumerate() {
            let id = core.state.lineages[i].id as usize;
            let spots = match mode {
                PlaceMode::Default => vec![Q[id % 4]],
                PlaceMode::Explicit(cells) => cells.clone(),
                PlaceMode::Random { k } => (0..*k)
                    .map(|_| {
                        let u = core.rng[3].next_u64();
                        ((u % 64) as u16, ((u >> 8) % 64) as u16)
                    })
                    .collect(),
            };
            for (x, y) in spots {
                if x >= W || y >= W {
                    return Err("cell out of range".into());
                }
                let mut idx = y as usize * 64 + x as usize;
                if matches!(mode, PlaceMode::Random { .. }) {
                    let start = idx;
                    while taken[idx] {
                        idx = (idx + 1) % N;
                        if idx == start {
                            return Err("no free cell".into());
                        }
                    }
                } else if taken[idx] {
                    return Err("duplicate cell".into());
                }
                taken[idx] = true;
                core.state.grid.cells[idx].biomass[id] = *mass;
            }
        }
        core.initial_mass = core.total_mass();
        core.sync_life_slots();
        Ok(core)
    }
}
