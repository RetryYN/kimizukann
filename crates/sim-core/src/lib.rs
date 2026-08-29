//! D1 one-cell closed-system core.

use kimizukann_sim_types::{
    CellState, ConversionRule, Fixed, GridState, InvariantReport, LineageParams, NumericError,
    Pool, Seed, StateHash, Thresholds, TickPhase, WorldState, FIXED_SCALE,
};
use sha2::{Digest, Sha256};

pub mod fixed {
    use kimizukann_sim_types::{ConversionRule, Fixed, NumericError, Pool, FIXED_SCALE};

    pub fn add(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        a.checked_add(b).ok_or(NumericError::OverflowI64)
    }
    pub fn sub_nonnegative(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        if b > a {
            return Err(NumericError::Negative);
        }
        Ok(a - b)
    }
    pub fn mul(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        let v = (a as i128)
            .checked_mul(b as i128)
            .ok_or(NumericError::OverflowI128)?;
        let q = v / FIXED_SCALE as i128;
        i64::try_from(q).map_err(|_| NumericError::OverflowI64)
    }
    pub fn div(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        if b == 0 {
            return Err(NumericError::Negative);
        }
        let v = (a as i128)
            .checked_mul(FIXED_SCALE as i128)
            .ok_or(NumericError::OverflowI128)?;
        i64::try_from(v / b as i128).map_err(|_| NumericError::OverflowI64)
    }
    pub fn split_output(input: Fixed, coefficient: Fixed) -> Result<(Fixed, Fixed), NumericError> {
        let rule = ConversionRule {
            from: Pool::Nutrient,
            to: Pool::Biomass,
            coefficient,
            remainder_to: Pool::Biomass,
        };
        split_output_with_rule(input, &rule, FIXED_SCALE - coefficient)
    }
    pub fn split_output_with_rule(
        input: Fixed,
        rule: &ConversionRule,
        waste_coefficient: Fixed,
    ) -> Result<(Fixed, Fixed), NumericError> {
        let primary = mul(input, rule.coefficient)?;
        let secondary = mul(input, waste_coefficient)?;
        let remainder = input - primary - secondary;
        match rule.remainder_to {
            Pool::Biomass => Ok((primary + remainder, secondary)),
            Pool::Waste => Ok((primary, secondary + remainder)),
            _ => Ok((primary, secondary)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Xoshiro256StarStar {
    state: [u64; 4],
}
impl Xoshiro256StarStar {
    pub fn from_seed(seed: u64) -> Self {
        let mut x = seed;
        let mut s = [0; 4];
        for v in &mut s {
            x = x.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            *v = z ^ (z >> 31);
        }
        Self { state: s }
    }
    pub fn next_u64(&mut self) -> u64 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }
    pub fn words(&self) -> [u64; 4] {
        self.state
    }
}

#[derive(Debug, Clone)]
pub struct SimCore {
    pub state: WorldState,
    pub seed: Seed,
    initial_mass: Fixed,
    pub rng: [Xoshiro256StarStar; 4],
    pub thresholds: Thresholds,
    pub model_version: String,
    /// Nutrient / carcass / waste / unused. Initial hypothesis 0.05 / neighbor / tick.
    pub diffusion_coefficients: [Fixed; 4],
}

impl SimCore {
    pub fn one_cell(
        seed: u64,
        nutrient: Fixed,
        biomass: Fixed,
        lineages: Vec<LineageParams>,
    ) -> Self {
        Self::try_one_cell(seed, nutrient, biomass, lineages).expect("invalid lineages")
    }
    pub fn try_one_cell(
        seed: u64,
        nutrient: Fixed,
        biomass: Fixed,
        mut lineages: Vec<LineageParams>,
    ) -> Result<Self, String> {
        lineages.sort_by_key(|l| l.id);
        if lineages.windows(2).any(|w| w[0].id == w[1].id) {
            return Err("duplicate lineage id".into());
        }
        let mut cell = CellState {
            nutrient,
            biomass: [0; 8],
            carcass: 0,
            waste: 0,
            energy: [FIXED_SCALE / 2; 8],
            occupancy_peak: 0,
        };
        if !lineages.is_empty() {
            cell.biomass[lineages[0].id as usize] = biomass;
        }
        let initial_mass = nutrient + biomass;
        let state = WorldState {
            tick: 0,
            grid: GridState {
                width: 1,
                height: 1,
                cells: vec![cell],
            },
            lineages,
        };
        Ok(Self {
            state,
            seed: Seed(seed),
            initial_mass,
            rng: [
                Xoshiro256StarStar::from_seed(seed ^ 0),
                Xoshiro256StarStar::from_seed(seed ^ 1),
                Xoshiro256StarStar::from_seed(seed ^ 2),
                Xoshiro256StarStar::from_seed(seed ^ 3),
            ],
            thresholds: Thresholds {
                base_intake: 100_000,
                base_maintenance: 10_000,
                epsilon: 1,
                fixed_share: 700_000,
                fixed_ticks: 200,
                coexist_share: 150_000,
                max_ticks: 2_000,
                waste_toxic_threshold: 100_000,
                toxin_maintenance_multiplier: 1_400_000,
                occupancy_threshold: FIXED_SCALE,
                vacant_nutrient_threshold: 100_000,
            },
            model_version: "d1-v1;prng=xoshiro256ss-v1;hash=sha256-v1".into(),
            diffusion_coefficients: [50_000; 4],
        })
    }

    pub fn try_grid(
        width: u16,
        height: u16,
        seed: u64,
        cells: Vec<CellState>,
        lineages: Vec<LineageParams>,
    ) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("grid dimension is 0".into());
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .ok_or("grid overflow")?;
        if cells.len() != expected {
            return Err("cells len != width*height".into());
        }
        let mut core = Self::try_one_cell(seed, 0, 0, lineages)?;
        core.state.grid = GridState {
            width,
            height,
            cells,
        };
        core.initial_mass = core.total_mass();
        Ok(core)
    }

    pub fn total_mass(&self) -> Fixed {
        self.state
            .grid
            .cells
            .iter()
            .map(|c| c.nutrient + c.biomass.iter().sum::<Fixed>() + c.carcass + c.waste)
            .sum()
    }

    pub fn apply_phase(&mut self, phase: TickPhase) -> Result<(), String> {
        match phase {
            TickPhase::Diffuse => self.diffuse(),
            TickPhase::Intake => self.intake(),
            TickPhase::Maintenance => self.maintenance(),
            TickPhase::StarvationAndDeath => self.starvation_and_death(),
            TickPhase::Reproduction => self.reproduction(),
            TickPhase::Emission => self.emission(),
            TickPhase::Occupancy => self.occupancy(),
        }
    }

    /// NESW. Filled in the grid-generalization commit; stub returns no neighbors.
    pub fn neighbor_indices(_width: u16, _height: u16, _index: usize) -> [Option<usize>; 4] {
        [None; 4]
    }

    /// Static 4x4 tiles. 64x64 -> 16x16 cells per tile, id = (row/16)*4 + (col/16).
    pub fn static_region_id(width: u16, height: u16, index: usize) -> u8 {
        let w = width as usize;
        let h = height as usize;
        if w == 0 || h == 0 {
            return 0;
        }
        let x = index % w;
        let y = index / w;
        let tile_w = (w / 4).max(1);
        let tile_h = (h / 4).max(1);
        let col = (x / tile_w).min(3);
        let row = (y / tile_h).min(3);
        (row * 4 + col) as u8
    }

    pub fn outflow_amount(pool: Fixed, coeff: Fixed) -> Result<Fixed, NumericError> {
        let v = (pool as i128)
            .checked_mul(coeff as i128)
            .ok_or(NumericError::OverflowI128)?;
        i64::try_from(v / FIXED_SCALE as i128).map_err(|_| NumericError::OverflowI64)
    }

    pub fn verify_suite_d2() -> (bool, bool) {
        (false, false)
    }

    pub fn step(&mut self, ticks: u32) -> Result<(), String> {
        for _ in 0..ticks {
            self.tick_once()?;
        }
        Ok(())
    }

    fn tick_once(&mut self) -> Result<(), String> {
        // Seven phases are explicit even though diffusion is a one-cell no-op in D1.
        self.diffuse()?;
        self.intake()?;
        self.maintenance()?;
        self.starvation_and_death()?;
        self.reproduction()?;
        self.emission()?;
        self.occupancy()?;
        self.state.tick = self.state.tick.checked_add(1).ok_or("tick overflow")?;
        Ok(())
    }
    fn diffuse(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn intake(&mut self) -> Result<(), String> {
        let cell = &mut self.state.grid.cells[0];
        for lineage in &self.state.lineages {
            let id = lineage.id as usize;
            if !lineage.tags.use_nutrient || cell.nutrient <= 0 {
                continue;
            }
            let amount = cell.nutrient.min(
                fixed::mul(self.thresholds.base_intake, lineage.traits.intake)
                    .map_err(|e| format!("intake: {e:?}"))?,
            );
            let rule = ConversionRule {
                from: Pool::Nutrient,
                to: Pool::Biomass,
                coefficient: 700_000,
                remainder_to: Pool::Biomass,
            };
            let (to_biomass, to_waste) = fixed::split_output_with_rule(amount, &rule, 300_000)
                .map_err(|e| format!("intake: {e:?}"))?;
            cell.nutrient -= amount;
            cell.biomass[id] =
                fixed::add(cell.biomass[id], to_biomass).map_err(|e| format!("biomass: {e:?}"))?;
            cell.waste = fixed::add(cell.waste, to_waste).map_err(|e| format!("waste: {e:?}"))?;
            cell.energy[id] = cell.energy[id].saturating_add(to_biomass).min(FIXED_SCALE);
        }
        Ok(())
    }
    fn maintenance(&mut self) -> Result<(), String> {
        let cell = &mut self.state.grid.cells[0];
        for lineage in &self.state.lineages {
            let id = lineage.id as usize;
            let mut cost = fixed::mul(
                self.thresholds.base_maintenance,
                lineage.traits.maintenance_cost,
            )
            .map_err(|e| format!("cost: {e:?}"))?;
            if lineage.tags.toxin_sensitive && cell.waste > self.thresholds.waste_toxic_threshold {
                cost = fixed::mul(cost, self.thresholds.toxin_maintenance_multiplier)
                    .map_err(|e| format!("toxin: {e:?}"))?;
            }
            let cost = cost.max(1);
            if cell.energy[id] >= cost {
                cell.energy[id] -= cost;
            } else {
                cell.energy[id] = 0;
            }
        }
        Ok(())
    }
    fn starvation_and_death(&mut self) -> Result<(), String> {
        let cell = &mut self.state.grid.cells[0];
        for lineage in &self.state.lineages {
            let id = lineage.id as usize;
            let cost = fixed::mul(
                self.thresholds.base_maintenance,
                lineage.traits.maintenance_cost,
            )
            .map_err(|e| format!("cost: {e:?}"))?;
            if cell.energy[id] < cost && cell.biomass[id] > 0 {
                let loss = cell.biomass[id].min(cost - cell.energy[id]);
                cell.biomass[id] -= loss;
                cell.carcass =
                    fixed::add(cell.carcass, loss).map_err(|e| format!("carcass: {e:?}"))?;
            }
            if cell.biomass[id] > 0 && cell.biomass[id] < lineage.mortality_threshold {
                let loss = cell.biomass[id];
                cell.biomass[id] = 0;
                cell.carcass =
                    fixed::add(cell.carcass, loss).map_err(|e| format!("death: {e:?}"))?;
            }
        }
        Ok(())
    }
    fn reproduction(&mut self) -> Result<(), String> {
        let cell = &mut self.state.grid.cells[0];
        for lineage in &self.state.lineages {
            let id = lineage.id as usize;
            let cost = fixed::mul(
                self.thresholds.base_maintenance,
                lineage.traits.maintenance_cost,
            )
            .map_err(|e| format!("cost: {e:?}"))?;
            let threshold = fixed::mul(cost * 2, lineage.traits.reproduction)
                .map_err(|e| format!("reproduction: {e:?}"))?;
            if cell.energy[id] > threshold && cell.nutrient > 0 {
                let gain = ((cell.energy[id] - threshold) / 2).min(cell.nutrient);
                cell.energy[id] -= gain;
                cell.nutrient -= gain;
                cell.biomass[id] = fixed::add(cell.biomass[id], gain)
                    .map_err(|e| format!("reproduction: {e:?}"))?;
            }
        }
        Ok(())
    }
    fn emission(&mut self) -> Result<(), String> {
        let cell = &mut self.state.grid.cells[0];
        for lineage in &self.state.lineages {
            let id = lineage.id as usize;
            let amount = cell.biomass[id].min(lineage.waste_emission.max(0));
            cell.biomass[id] -= amount;
            cell.waste = fixed::add(cell.waste, amount).map_err(|e| format!("emission: {e:?}"))?;
        }
        Ok(())
    }
    fn occupancy(&mut self) -> Result<(), String> {
        let cell = &mut self.state.grid.cells[0];
        let biomass: Fixed = cell.biomass.iter().sum();
        if biomass >= self.thresholds.occupancy_threshold {
            cell.occupancy_peak = FIXED_SCALE;
        } else {
            cell.occupancy_peak = fixed::mul(cell.occupancy_peak, 995_000)
                .map_err(|e| format!("occupancy: {e:?}"))?;
        }
        Ok(())
    }

    pub fn invariant_report(&self) -> InvariantReport {
        let cell = &self.state.grid.cells[0];
        let biomass: Fixed = cell.biomass.iter().sum();
        let mass = cell.nutrient + biomass + cell.carcass + cell.waste;
        let non_negative = cell.nutrient >= 0
            && cell.carcass >= 0
            && cell.waste >= 0
            && cell.biomass.iter().all(|v| *v >= 0)
            && cell.energy.iter().all(|v| *v >= 0 && *v <= FIXED_SCALE);
        InvariantReport {
            mass_ok: mass == self.initial_mass,
            energy_ok: cell.energy.iter().all(|v| *v >= 0 && *v <= FIXED_SCALE),
            non_negative,
            message: format!("mass={mass} initial={}", self.initial_mass),
        }
    }

    pub fn state_hash(&self) -> StateHash {
        let mut h = Sha256::new();
        h.update(self.state.tick.to_le_bytes());
        h.update(self.seed.0.to_le_bytes());
        h.update(self.state.grid.width.to_le_bytes());
        h.update(self.state.grid.height.to_le_bytes());
        h.update(self.model_version.as_bytes());
        for stream in &self.rng {
            for word in stream.words() {
                h.update(word.to_le_bytes());
            }
        }
        for c in &self.state.grid.cells {
            h.update(c.nutrient.to_le_bytes());
            for v in c.biomass {
                h.update(v.to_le_bytes());
            }
            h.update(c.carcass.to_le_bytes());
            h.update(c.waste.to_le_bytes());
            for v in c.energy {
                h.update(v.to_le_bytes());
            }
            h.update(c.occupancy_peak.to_le_bytes());
        }
        StateHash(h.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimizukann_sim_types::{LineageParams, MechanismTags, TraitVector};
    fn lineage() -> LineageParams {
        LineageParams {
            id: 0,
            traits: TraitVector {
                movement: FIXED_SCALE,
                intake: FIXED_SCALE,
                conversion: FIXED_SCALE,
                maintenance_cost: FIXED_SCALE,
                reproduction: FIXED_SCALE,
            },
            tags: MechanismTags {
                use_nutrient: true,
                ..MechanismTags::default()
            },
            mortality_threshold: 1,
            waste_emission: 1,
        }
    }
    #[test]
    fn fixed_rounding_and_remainder() {
        assert_eq!(fixed::mul(3, 500_000).unwrap(), 1);
        assert_eq!(fixed::split_output(3, 500_000).unwrap(), (2, 1));
    }
    #[test]
    fn conservation_and_nonnegative() {
        let mut s = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage()]);
        let before = s.state.grid.cells[0].clone();
        s.step(2000).unwrap();
        let after = &s.state.grid.cells[0];
        let r = s.invariant_report();
        assert!(r.mass_ok && r.energy_ok && r.non_negative);
        assert!(
            after.nutrient != before.nutrient
                || after.waste != before.waste
                || after.biomass != before.biomass
        );
    }
    #[test]
    fn hash_golden() {
        let s = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage()]);
        let expected = "453b41f19db8e3010258c3f8ed964b475333b06b0c27859ab9105be3ddcb6a0a";
        let actual: String = s
            .state_hash()
            .0
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(actual, expected);
    }
}
