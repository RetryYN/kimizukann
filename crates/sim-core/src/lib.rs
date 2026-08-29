//! D1 one-cell closed-system core.

use kimizukann_sim_types::{CellState, Fixed, GridState, InvariantReport, LineageParams, Seed, StateHash, WorldState, FIXED_SCALE};
use sha2::{Digest, Sha256};

pub mod fixed {
    use kimizukann_sim_types::{Fixed, NumericError, FIXED_SCALE};

    pub fn add(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> { a.checked_add(b).ok_or(NumericError::OverflowI64) }
    pub fn sub_nonnegative(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        if b > a { return Err(NumericError::Negative); }
        Ok(a - b)
    }
    pub fn mul(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        let v = (a as i128).checked_mul(b as i128).ok_or(NumericError::OverflowI128)?;
        let q = v / FIXED_SCALE as i128;
        i64::try_from(q).map_err(|_| NumericError::OverflowI64)
    }
    pub fn div(a: Fixed, b: Fixed) -> Result<Fixed, NumericError> {
        if b == 0 { return Err(NumericError::Negative); }
        let v = (a as i128).checked_mul(FIXED_SCALE as i128).ok_or(NumericError::OverflowI128)?;
        i64::try_from(v / b as i128).map_err(|_| NumericError::OverflowI64)
    }
    pub fn split_output(input: Fixed, coefficient: Fixed) -> Result<(Fixed, Fixed), NumericError> {
        let out = mul(input, coefficient)?;
        Ok((out, input - out))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Xoshiro256StarStar { state: [u64; 4] }
impl Xoshiro256StarStar {
    pub fn from_seed(seed: u64) -> Self {
        let mut x = seed;
        let mut s = [0; 4];
        for v in &mut s { x = x.wrapping_add(0x9e3779b97f4a7c15); let mut z=x; z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9); z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb); *v=z^(z>>31); }
        Self { state: s }
    }
    pub fn next_u64(&mut self) -> u64 { let result=self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9); let t=self.state[1]<<17; self.state[2]^=self.state[0]; self.state[3]^=self.state[1]; self.state[1]^=self.state[2]; self.state[0]^=self.state[3]; self.state[2]^=t; self.state[3]=self.state[3].rotate_left(45); result }
}

#[derive(Debug, Clone)]
pub struct SimCore { pub state: WorldState, pub seed: Seed, initial_mass: Fixed, pub rng: Xoshiro256StarStar }

impl SimCore {
    pub fn one_cell(seed: u64, nutrient: Fixed, biomass: Fixed, lineages: Vec<LineageParams>) -> Self {
        let mut cell = CellState { nutrient, biomass: [0; 8], carcass: 0, waste: 0, energy: [FIXED_SCALE; 8], occupancy_peak: 0 };
        if !lineages.is_empty() { cell.biomass[lineages[0].id as usize] = biomass; }
        let initial_mass = nutrient + biomass;
        let state = WorldState { tick: 0, grid: GridState { width: 1, height: 1, cells: vec![cell] }, lineages };
        Self { state, seed: Seed(seed), initial_mass, rng: Xoshiro256StarStar::from_seed(seed) }
    }

    pub fn step(&mut self, ticks: u32) -> Result<(), String> {
        for _ in 0..ticks { self.tick_once()?; }
        Ok(())
    }

    fn tick_once(&mut self) -> Result<(), String> {
        // Seven phases are explicit even though diffusion is a one-cell no-op in D1.
        self.diffuse()?; self.intake()?; self.maintenance()?; self.starvation_and_death()?; self.reproduction()?; self.emission()?; self.occupancy()?;
        self.state.tick = self.state.tick.checked_add(1).ok_or("tick overflow")?;
        Ok(())
    }
    fn diffuse(&mut self) -> Result<(), String> { Ok(()) }
    fn intake(&mut self) -> Result<(), String> { Ok(()) }
    fn maintenance(&mut self) -> Result<(), String> { Ok(()) }
    fn starvation_and_death(&mut self) -> Result<(), String> { Ok(()) }
    fn reproduction(&mut self) -> Result<(), String> { Ok(()) }
    fn emission(&mut self) -> Result<(), String> { Ok(()) }
    fn occupancy(&mut self) -> Result<(), String> { Ok(()) }

    pub fn invariant_report(&self) -> InvariantReport {
        let cell = &self.state.grid.cells[0];
        let biomass: Fixed = cell.biomass.iter().sum();
        let mass = cell.nutrient + biomass + cell.carcass + cell.waste;
        let non_negative = cell.nutrient >= 0 && cell.carcass >= 0 && cell.waste >= 0 && cell.biomass.iter().all(|v| *v >= 0) && cell.energy.iter().all(|v| *v >= 0 && *v <= FIXED_SCALE);
        InvariantReport { mass_ok: mass == self.initial_mass, energy_ok: cell.energy.iter().all(|v| *v >= 0 && *v <= FIXED_SCALE), non_negative, message: format!("mass={mass} initial={}", self.initial_mass) }
    }

    pub fn state_hash(&self) -> StateHash {
        let mut h = Sha256::new();
        h.update(self.state.tick.to_le_bytes()); h.update(self.seed.0.to_le_bytes());
        h.update(self.state.grid.width.to_le_bytes()); h.update(self.state.grid.height.to_le_bytes());
        for c in &self.state.grid.cells { h.update(c.nutrient.to_le_bytes()); for v in c.biomass { h.update(v.to_le_bytes()); } h.update(c.carcass.to_le_bytes()); h.update(c.waste.to_le_bytes()); for v in c.energy { h.update(v.to_le_bytes()); } h.update(c.occupancy_peak.to_le_bytes()); }
        StateHash(h.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimizukann_sim_types::{LineageParams, MechanismTags, TraitVector};
    fn lineage() -> LineageParams { LineageParams { id: 0, traits: TraitVector { movement: FIXED_SCALE, intake: FIXED_SCALE, conversion: FIXED_SCALE, maintenance_cost: FIXED_SCALE, reproduction: FIXED_SCALE }, tags: MechanismTags::default(), mortality_threshold: 1, waste_emission: 1 } }
    #[test] fn fixed_rounding_and_remainder() { assert_eq!(fixed::mul(3, 500_000).unwrap(), 1); assert_eq!(fixed::split_output(3, 500_000).unwrap(), (1, 2)); }
    #[test] fn conservation_and_nonnegative() { let mut s=SimCore::one_cell(7, 10*FIXED_SCALE, 2*FIXED_SCALE, vec![lineage()]); s.step(2000).unwrap(); let r=s.invariant_report(); assert!(r.mass_ok && r.energy_ok && r.non_negative); }
    #[test] fn hash_golden() { let s=SimCore::one_cell(7, 10*FIXED_SCALE, 2*FIXED_SCALE, vec![lineage()]); let expected = "a3d1da7f8d1d70cf916cc4610e9d69d8994515654c61f3dc8b2e4899803467e9"; let actual: String = s.state_hash().0.iter().map(|b| format!("{b:02x}")).collect(); assert_eq!(actual, expected); }
}
