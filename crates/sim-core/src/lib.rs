//! D1 one-cell closed-system core.

use kimizukann_sim_types::{
    CellState, ConversionRule, Fixed, GridState, InvariantReport, LineageParams, NumericError,
    Pool, ReasonCode, Seed, StateHash, Thresholds, TickPhase, WorldState, FIXED_SCALE,
};
use sha2::{Digest, Sha256};

mod ledger;
pub use ledger::{fold_region_records, LedgerRecord};

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

#[derive(Debug, Clone, Default)]
struct DiffuseScratch {
    d_n: Vec<i64>,
    d_c: Vec<i64>,
    d_w: Vec<i64>,
    d_b: Vec<[i64; 8]>,
    neighbors: Vec<[Option<usize>; 4]>,
    cached_w: u16,
    cached_h: u16,
}

impl DiffuseScratch {
    fn prepare(&mut self, w: u16, h: u16, n: usize) {
        if self.d_n.len() != n {
            self.d_n.resize(n, 0);
            self.d_c.resize(n, 0);
            self.d_w.resize(n, 0);
            self.d_b.resize(n, [0; 8]);
        } else {
            self.d_n.fill(0);
            self.d_c.fill(0);
            self.d_w.fill(0);
            self.d_b.fill([0; 8]);
        }
        if self.cached_w != w || self.cached_h != h || self.neighbors.len() != n {
            self.neighbors.resize(n, [None; 4]);
            for (i, slot) in self.neighbors.iter_mut().enumerate() {
                *slot = SimCore::neighbor_indices(w, h, i);
            }
            self.cached_w = w;
            self.cached_h = h;
        }
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
    pub diffusion_coefficients: [Fixed; 4],
    scratch: DiffuseScratch,
    pub mass_ledger: Vec<LedgerRecord>,
    pub energy_ledger: Vec<LedgerRecord>,
    pub life: Vec<[u8; 8]>,
    deficit: Vec<[Fixed; 8]>,
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
        let mut core = Self {
            state,
            seed: Seed(seed),
            initial_mass,
            rng: [
                Xoshiro256StarStar::from_seed(seed),
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
            model_version: "d3-v1;prng=xoshiro256ss-v1;hash=sha256-v1".into(),
            diffusion_coefficients: [50_000; 4],
            scratch: DiffuseScratch::default(),
            mass_ledger: Vec::new(),
            energy_ledger: Vec::new(),
            life: vec![[0; 8]],
            deficit: vec![[0; 8]],
        };
        core.sync_life_slots();
        Ok(core)
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
        core.sync_life_slots();
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
            _ => Err("phase".into()),
        }
    }

    fn sync_life_slots(&mut self) {
        let n = self.state.grid.cells.len();
        self.life.resize(n, [0; 8]);
        self.deficit.resize(n, [0; 8]);
        for (i, cell) in self.state.grid.cells.iter().enumerate() {
            for id in 0..8 {
                if self.life[i][id] == 0 && cell.biomass[id] > 0 {
                    self.life[i][id] = 1;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push_row(
        rows: &mut Vec<LedgerRecord>,
        tick: u32,
        region_id: u8,
        lineage: u8,
        reason: ReasonCode,
        from: Pool,
        to: Pool,
        amount: Fixed,
    ) {
        if amount > 0 {
            rows.push(LedgerRecord {
                tick,
                region_id,
                lineage,
                reason,
                from_pool: from,
                to_pool: to,
                amount,
            });
        }
    }

    pub fn neighbor_indices(width: u16, height: u16, index: usize) -> [Option<usize>; 4] {
        let w = width as usize;
        let h = height as usize;
        if w == 0 || index >= w.saturating_mul(h) {
            return [None; 4];
        }
        let x = index % w;
        let y = index / w;
        [
            if y > 0 { Some(index - w) } else { None },
            if x + 1 < w { Some(index + 1) } else { None },
            if y + 1 < h { Some(index + w) } else { None },
            if x > 0 { Some(index - 1) } else { None },
        ]
    }

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
        let blank = CellState {
            nutrient: 0,
            biomass: [0; 8],
            carcass: 0,
            waste: 0,
            energy: [FIXED_SCALE / 2; 8],
            occupancy_peak: 0,
        };
        let conservation = (|| -> Result<bool, String> {
            let mut cells = vec![blank.clone(); 64 * 64];
            cells[0].nutrient = FIXED_SCALE;
            let mut s = Self::try_grid(64, 64, 11, cells, vec![])?;
            let mass = s.total_mass();
            for _ in 0..2_000 {
                s.apply_phase(TickPhase::Diffuse)?;
            }
            Ok(s.total_mass() == mass)
        })()
        .unwrap_or(false);
        let symmetry = (|| -> Result<bool, String> {
            let mut left = vec![blank.clone(); 2];
            left[0].nutrient = FIXED_SCALE;
            let mut right = vec![blank.clone(); 2];
            right[1].nutrient = FIXED_SCALE;
            let mut a = Self::try_grid(2, 1, 13, left, vec![])?;
            let mut b = Self::try_grid(2, 1, 13, right, vec![])?;
            a.apply_phase(TickPhase::Diffuse)?;
            b.apply_phase(TickPhase::Diffuse)?;
            Ok(
                a.state.grid.cells[0].nutrient == b.state.grid.cells[1].nutrient
                    && a.state.grid.cells[1].nutrient == b.state.grid.cells[0].nutrient,
            )
        })()
        .unwrap_or(false);
        (conservation, symmetry)
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
        fold_region_records(&mut self.mass_ledger);
        fold_region_records(&mut self.energy_ledger);
        self.state.tick = self.state.tick.checked_add(1).ok_or("tick overflow")?;
        Ok(())
    }
    fn next_pool(pool: Fixed, delta: i64) -> Result<Fixed, String> {
        let next = pool
            .checked_add(delta)
            .ok_or_else(|| format!("diffuse: {:?}", NumericError::OverflowI64))?;
        if next < 0 {
            return Err(format!("diffuse: {:?}", NumericError::Negative));
        }
        Ok(next)
    }

    fn diffuse(&mut self) -> Result<(), String> {
        let (w, h) = (self.state.grid.width, self.state.grid.height);
        let n = self.state.grid.cells.len();
        self.scratch.prepare(w, h, n);
        let coeffs = self.diffusion_coefficients;
        let mut move_by = [0; 8];
        for lineage in &self.state.lineages {
            if (lineage.id as usize) < 8 {
                move_by[lineage.id as usize] = lineage.traits.movement;
            }
        }
        let any_move = move_by.iter().any(|&m| m != 0);
        for (i, cell) in self.state.grid.cells.iter().enumerate() {
            let send = |pool: Fixed, coeff: Fixed| {
                Self::outflow_amount(pool, coeff).map_err(|e| format!("diffuse: {e:?}"))
            };
            let n_out = send(cell.nutrient, coeffs[0])?;
            let c_out = send(cell.carcass, coeffs[1])?;
            let w_out = send(cell.waste, coeffs[2])?;
            let mut b_out = [0; 8];
            if any_move {
                for (id, (&mv, &pool)) in move_by.iter().zip(cell.biomass.iter()).enumerate() {
                    if mv != 0 {
                        b_out[id] = send(pool, mv)?;
                    }
                }
            }
            for dest in self.scratch.neighbors[i].into_iter().flatten() {
                self.scratch.d_n[i] -= n_out;
                self.scratch.d_n[dest] += n_out;
                self.scratch.d_c[i] -= c_out;
                self.scratch.d_c[dest] += c_out;
                self.scratch.d_w[i] -= w_out;
                self.scratch.d_w[dest] += w_out;
                if any_move {
                    for (id, out) in b_out.iter().enumerate() {
                        if *out == 0 {
                            continue;
                        }
                        self.scratch.d_b[i][id] -= *out;
                        self.scratch.d_b[dest][id] += *out;
                    }
                }
            }
        }
        for (i, cell) in self.state.grid.cells.iter().enumerate() {
            Self::next_pool(cell.nutrient, self.scratch.d_n[i])?;
            Self::next_pool(cell.carcass, self.scratch.d_c[i])?;
            Self::next_pool(cell.waste, self.scratch.d_w[i])?;
            for (slot, delta) in cell.biomass.iter().zip(self.scratch.d_b[i].iter()) {
                Self::next_pool(*slot, *delta)?;
            }
        }
        for (i, cell) in self.state.grid.cells.iter_mut().enumerate() {
            cell.nutrient = Self::next_pool(cell.nutrient, self.scratch.d_n[i])?;
            cell.carcass = Self::next_pool(cell.carcass, self.scratch.d_c[i])?;
            cell.waste = Self::next_pool(cell.waste, self.scratch.d_w[i])?;
            for (slot, delta) in cell.biomass.iter_mut().zip(self.scratch.d_b[i].iter()) {
                *slot = Self::next_pool(*slot, *delta)?;
            }
        }
        Ok(())
    }

    /// D2-Q1 boundary: cell-grain Diffusion rows are not retained.
    /// Tick-end callers fold region aggregates here (no-op until ledger wiring).
    pub fn fold_diffuse_region_aggregates<F>(&self, _fold: F)
    where
        F: FnMut(u8, Pool, Pool, Fixed),
    {
    }
    fn intake(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        let tick = self.state.tick;
        let (w, h) = (self.state.grid.width, self.state.grid.height);
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                let id = lineage.id as usize;
                if id >= 8 || self.life[cell_i][id] == 0 {
                    continue;
                }
                let region_id = Self::static_region_id(w, h, cell_i);
                for (on, pool, bio_c, waste_c) in [
                    (lineage.tags.use_nutrient, Pool::Nutrient, 700_000, 300_000),
                    (lineage.tags.use_carcass, Pool::Carcass, 500_000, 500_000),
                    (lineage.tags.use_waste, Pool::Waste, 500_000, 500_000),
                ] {
                    if !on {
                        continue;
                    }
                    let available = match pool {
                        Pool::Nutrient => self.state.grid.cells[cell_i].nutrient,
                        Pool::Carcass => self.state.grid.cells[cell_i].carcass,
                        Pool::Waste => self.state.grid.cells[cell_i].waste,
                        Pool::Biomass => 0,
                    };
                    if available <= 0 {
                        continue;
                    }
                    let amount = available.min(
                        fixed::mul(self.thresholds.base_intake, lineage.traits.intake)
                            .map_err(|e| format!("intake: {e:?}"))?,
                    );
                    let rule = ConversionRule {
                        from: pool,
                        to: Pool::Biomass,
                        coefficient: bio_c,
                        remainder_to: Pool::Biomass,
                    };
                    let (to_biomass, to_waste) = if pool == Pool::Nutrient {
                        fixed::split_output_with_rule(amount, &rule, 300_000)
                    } else {
                        fixed::split_output_with_rule(amount, &rule, waste_c)
                    }
                    .map_err(|e| format!("intake: {e:?}"))?;
                    let cell = &mut self.state.grid.cells[cell_i];
                    match pool {
                        Pool::Nutrient => cell.nutrient -= amount,
                        Pool::Carcass => cell.carcass -= amount,
                        Pool::Waste => cell.waste -= amount,
                        Pool::Biomass => {}
                    }
                    cell.biomass[id] = fixed::add(cell.biomass[id], to_biomass)
                        .map_err(|e| format!("biomass: {e:?}"))?;
                    cell.waste =
                        fixed::add(cell.waste, to_waste).map_err(|e| format!("waste: {e:?}"))?;
                    let next = cell.energy[id].saturating_add(amount);
                    let heat = next.saturating_sub(FIXED_SCALE);
                    cell.energy[id] = next.min(FIXED_SCALE);
                    Self::push_row(
                        &mut self.energy_ledger,
                        tick,
                        region_id,
                        lineage.id,
                        ReasonCode::Intake,
                        pool,
                        Pool::Biomass,
                        amount - heat,
                    );
                    Self::push_row(
                        &mut self.energy_ledger,
                        tick,
                        region_id,
                        lineage.id,
                        ReasonCode::Intake,
                        Pool::Biomass,
                        Pool::Waste,
                        heat,
                    );
                    Self::push_row(
                        &mut self.mass_ledger,
                        tick,
                        region_id,
                        lineage.id,
                        ReasonCode::Intake,
                        pool,
                        Pool::Biomass,
                        to_biomass,
                    );
                    Self::push_row(
                        &mut self.mass_ledger,
                        tick,
                        region_id,
                        lineage.id,
                        ReasonCode::Intake,
                        pool,
                        Pool::Waste,
                        to_waste,
                    );
                }
            }
        }
        Ok(())
    }
    fn maintenance(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        let tick = self.state.tick;
        let (w, h) = (self.state.grid.width, self.state.grid.height);
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                let id = lineage.id as usize;
                if id >= 8 || self.life[cell_i][id] == 0 {
                    continue;
                }
                let mut cost = fixed::mul(
                    self.thresholds.base_maintenance,
                    lineage.traits.maintenance_cost,
                )
                .map_err(|e| format!("cost: {e:?}"))?;
                if lineage.tags.toxin_sensitive
                    && self.state.grid.cells[cell_i].waste > self.thresholds.waste_toxic_threshold
                {
                    cost = fixed::mul(cost, self.thresholds.toxin_maintenance_multiplier)
                        .map_err(|e| format!("toxin: {e:?}"))?;
                }
                let cost = cost.max(1);
                let energy = self.state.grid.cells[cell_i].energy[id];
                let region_id = Self::static_region_id(w, h, cell_i);
                if energy >= cost {
                    self.state.grid.cells[cell_i].energy[id] = energy - cost;
                    Self::push_row(
                        &mut self.energy_ledger,
                        tick,
                        region_id,
                        lineage.id,
                        ReasonCode::Maintenance,
                        Pool::Biomass,
                        Pool::Waste,
                        cost,
                    );
                } else {
                    self.deficit[cell_i][id] = cost - energy;
                    self.state.grid.cells[cell_i].energy[id] = 0;
                    self.life[cell_i][id] = 2;
                    Self::push_row(
                        &mut self.energy_ledger,
                        tick,
                        region_id,
                        lineage.id,
                        ReasonCode::Maintenance,
                        Pool::Biomass,
                        Pool::Waste,
                        energy,
                    );
                }
            }
        }
        Ok(())
    }
    fn starvation_and_death(&mut self) -> Result<(), String> {
        let lineages = self.state.lineages.clone();
        for cell in &mut self.state.grid.cells {
            for lineage in &lineages {
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
        }
        Ok(())
    }
    fn reproduction(&mut self) -> Result<(), String> {
        let lineages = self.state.lineages.clone();
        for cell in &mut self.state.grid.cells {
            for lineage in &lineages {
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
        }
        Ok(())
    }
    fn emission(&mut self) -> Result<(), String> {
        let lineages = self.state.lineages.clone();
        for cell in &mut self.state.grid.cells {
            for lineage in &lineages {
                let id = lineage.id as usize;
                let amount = cell.biomass[id].min(lineage.waste_emission.max(0));
                cell.biomass[id] -= amount;
                cell.waste =
                    fixed::add(cell.waste, amount).map_err(|e| format!("emission: {e:?}"))?;
            }
        }
        Ok(())
    }
    fn occupancy(&mut self) -> Result<(), String> {
        let threshold = self.thresholds.occupancy_threshold;
        for cell in &mut self.state.grid.cells {
            let biomass: Fixed = cell.biomass.iter().sum();
            if biomass >= threshold {
                cell.occupancy_peak = FIXED_SCALE;
            } else {
                cell.occupancy_peak = fixed::mul(cell.occupancy_peak, 995_000)
                    .map_err(|e| format!("occupancy: {e:?}"))?;
            }
        }
        Ok(())
    }

    pub fn invariant_report(&self) -> InvariantReport {
        let mass = self.total_mass();
        let non_negative = self.state.grid.cells.iter().all(|c| {
            c.nutrient >= 0
                && c.carcass >= 0
                && c.waste >= 0
                && c.biomass.iter().all(|v| *v >= 0)
                && c.energy.iter().all(|v| *v >= 0 && *v <= FIXED_SCALE)
        });
        let energy_ok = self
            .state
            .grid
            .cells
            .iter()
            .all(|c| c.energy.iter().all(|v| *v >= 0 && *v <= FIXED_SCALE));
        InvariantReport {
            mass_ok: mass == self.initial_mass,
            energy_ok,
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
        let expected = "3c96754933c3e4ae5d412b64cbb89370e9172effb8274ac7009250ca39850d3c";
        let actual: String = s
            .state_hash()
            .0
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(actual, expected);
    }
    #[test]
    fn ut_d3_01_intake_order_and_heat() {
        let mut b = SimCore::one_cell(7, 100_000, FIXED_SCALE, vec![lineage()]);
        b.state.grid.cells[0].energy[0] = FIXED_SCALE - 10_000;
        b.apply_phase(TickPhase::Intake).unwrap();
        assert_eq!(b.state.grid.cells[0].energy[0], FIXED_SCALE);
        let mut c = SimCore::one_cell(7, 0, FIXED_SCALE, vec![lineage()]);
        c.state.grid.cells[0].energy[0] = 1;
        c.apply_phase(TickPhase::Maintenance).unwrap();
        assert_eq!(c.life[0][0], 2);
        assert!(c
            .energy_ledger
            .iter()
            .any(|r| r.reason == ReasonCode::Maintenance));
    }
}
