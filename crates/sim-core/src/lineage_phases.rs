use kimizukann_sim_types::{
    ConversionRule, Fixed, LineageParams, Pool, ReasonCode, TickPhase, FIXED_SCALE,
};

use super::{fixed, LedgerRecord, SimCore};

const LIFE_ABSENT: u8 = 0;
const LIFE_ALIVE: u8 = 1;
const LIFE_STARVING: u8 = 2;

impl SimCore {
    pub(crate) fn run_lineage_phase(&mut self, phase: TickPhase) -> Result<(), String> {
        match phase {
            TickPhase::Intake => self.lineage_intake(),
            TickPhase::Maintenance => self.lineage_maintenance(),
            TickPhase::StarvationAndDeath => self.lineage_starvation_and_death(),
            TickPhase::Reproduction => self.lineage_reproduction(),
            TickPhase::Emission => self.lineage_emission(),
            TickPhase::Occupancy => self.occupancy(),
            TickPhase::Diffuse => self.diffuse(),
        }
    }

    fn push_mass(
        &mut self,
        cell_index: usize,
        lineage: u8,
        reason: ReasonCode,
        from: Pool,
        to: Pool,
        amount: Fixed,
    ) {
        if amount <= 0 {
            return;
        }
        let region_id =
            Self::static_region_id(self.state.grid.width, self.state.grid.height, cell_index);
        self.mass_ledger.push(LedgerRecord {
            tick: self.state.tick,
            region_id,
            lineage,
            reason,
            from_pool: from,
            to_pool: to,
            amount,
        });
    }

    fn push_energy(
        &mut self,
        cell_index: usize,
        lineage: u8,
        reason: ReasonCode,
        from: Pool,
        to: Pool,
        amount: Fixed,
    ) {
        if amount <= 0 {
            return;
        }
        let region_id =
            Self::static_region_id(self.state.grid.width, self.state.grid.height, cell_index);
        self.energy_ledger.push(LedgerRecord {
            tick: self.state.tick,
            region_id,
            lineage,
            reason,
            from_pool: from,
            to_pool: to,
            amount,
        });
    }

    fn lineage_intake(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                self.intake_one(cell_i, lineage)?;
            }
        }
        Ok(())
    }

    fn intake_one(&mut self, cell_i: usize, lineage: &LineageParams) -> Result<(), String> {
        let id = lineage.id as usize;
        if id >= 8 || self.life[cell_i][id] == LIFE_ABSENT {
            return Ok(());
        }
        let substrates: [(bool, Pool); 3] = [
            (lineage.tags.use_nutrient, Pool::Nutrient),
            (lineage.tags.use_carcass, Pool::Carcass),
            (lineage.tags.use_waste, Pool::Waste),
        ];
        for (enabled, pool) in substrates {
            if !enabled {
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
            let cap = fixed::mul(self.thresholds.base_intake, lineage.traits.intake)
                .map_err(|e| format!("intake: {e:?}"))?;
            let take = available.min(cap);
            let (coeff_bio, coeff_waste) = match pool {
                Pool::Nutrient => (700_000, 300_000),
                _ => (500_000, 500_000),
            };
            let rule = ConversionRule {
                from: pool,
                to: Pool::Biomass,
                coefficient: coeff_bio,
                remainder_to: Pool::Biomass,
            };
            let (to_biomass, to_waste) = fixed::split_output_with_rule(take, &rule, coeff_waste)
                .map_err(|e| format!("intake: {e:?}"))?;
            let heat;
            {
                let cell = &mut self.state.grid.cells[cell_i];
                match pool {
                    Pool::Nutrient => cell.nutrient -= take,
                    Pool::Carcass => cell.carcass -= take,
                    Pool::Waste => cell.waste -= take,
                    Pool::Biomass => {}
                }
                cell.biomass[id] =
                    fixed::add(cell.biomass[id], to_biomass).map_err(|e| format!("biomass: {e:?}"))?;
                cell.waste = fixed::add(cell.waste, to_waste).map_err(|e| format!("waste: {e:?}"))?;
                let next_energy = cell.energy[id].saturating_add(take);
                heat = next_energy.saturating_sub(FIXED_SCALE);
                cell.energy[id] = next_energy.min(FIXED_SCALE);
            }
            self.push_energy(
                cell_i,
                lineage.id,
                ReasonCode::Intake,
                Pool::Nutrient,
                Pool::Biomass,
                take - heat,
            );
            self.push_energy(
                cell_i,
                lineage.id,
                ReasonCode::Intake,
                Pool::Biomass,
                Pool::Waste,
                heat,
            );
            self.push_mass(
                cell_i,
                lineage.id,
                ReasonCode::Intake,
                pool,
                Pool::Biomass,
                to_biomass,
            );
            self.push_mass(
                cell_i,
                lineage.id,
                ReasonCode::Intake,
                pool,
                Pool::Waste,
                to_waste,
            );
        }
        Ok(())
    }

    fn lineage_maintenance(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                let id = lineage.id as usize;
                if id >= 8 || self.life[cell_i][id] == LIFE_ABSENT {
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
                if energy >= cost {
                    self.state.grid.cells[cell_i].energy[id] = energy - cost;
                    self.push_energy(
                        cell_i,
                        lineage.id,
                        ReasonCode::Maintenance,
                        Pool::Biomass,
                        Pool::Waste,
                        cost,
                    );
                } else {
                    self.deficit[cell_i][id] = cost - energy;
                    self.state.grid.cells[cell_i].energy[id] = 0;
                    self.life[cell_i][id] = LIFE_STARVING;
                    self.push_energy(
                        cell_i,
                        lineage.id,
                        ReasonCode::Maintenance,
                        Pool::Biomass,
                        Pool::Waste,
                        energy,
                    );
                    self.push_energy(
                        cell_i,
                        lineage.id,
                        ReasonCode::Maintenance,
                        Pool::Waste,
                        Pool::Carcass,
                        cost - energy,
                    );
                }
            }
        }
        Ok(())
    }

    fn lineage_starvation_and_death(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                let id = lineage.id as usize;
                if id >= 8 || self.life[cell_i][id] != LIFE_STARVING {
                    continue;
                }
                let deficit = self.deficit[cell_i][id];
                let biomass = self.state.grid.cells[cell_i].biomass[id];
                let loss = biomass.min(deficit);
                if loss > 0 {
                    self.state.grid.cells[cell_i].biomass[id] -= loss;
                    self.state.grid.cells[cell_i].carcass = fixed::add(
                        self.state.grid.cells[cell_i].carcass,
                        loss,
                    )
                    .map_err(|e| format!("carcass: {e:?}"))?;
                    self.push_mass(
                        cell_i,
                        lineage.id,
                        ReasonCode::Starvation,
                        Pool::Biomass,
                        Pool::Carcass,
                        loss,
                    );
                }
                let left = self.state.grid.cells[cell_i].biomass[id];
                if left >= lineage.mortality_threshold {
                    self.life[cell_i][id] = LIFE_ALIVE;
                    self.deficit[cell_i][id] = 0;
                } else {
                    if left > 0 {
                        self.state.grid.cells[cell_i].biomass[id] = 0;
                        self.state.grid.cells[cell_i].carcass = fixed::add(
                            self.state.grid.cells[cell_i].carcass,
                            left,
                        )
                        .map_err(|e| format!("death: {e:?}"))?;
                        self.push_mass(
                            cell_i,
                            lineage.id,
                            ReasonCode::Death,
                            Pool::Biomass,
                            Pool::Carcass,
                            left,
                        );
                    }
                    self.life[cell_i][id] = LIFE_ABSENT;
                    self.deficit[cell_i][id] = 0;
                }
            }
        }
        Ok(())
    }

    fn lineage_reproduction(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                let id = lineage.id as usize;
                if id >= 8 || self.life[cell_i][id] != LIFE_ALIVE {
                    continue;
                }
                let cost = fixed::mul(
                    self.thresholds.base_maintenance,
                    lineage.traits.maintenance_cost,
                )
                .map_err(|e| format!("cost: {e:?}"))?;
                let guard = cost.saturating_mul(2);
                let energy = self.state.grid.cells[cell_i].energy[id];
                if energy <= guard {
                    continue;
                }
                let _draw = self.rng[1].next_u64();
                let surplus = (energy - guard) / 2;
                let gain = surplus.min(self.state.grid.cells[cell_i].nutrient);
                if gain <= 0 {
                    continue;
                }
                self.state.grid.cells[cell_i].energy[id] = energy - gain;
                self.state.grid.cells[cell_i].nutrient -= gain;
                self.state.grid.cells[cell_i].biomass[id] = fixed::add(
                    self.state.grid.cells[cell_i].biomass[id],
                    gain,
                )
                .map_err(|e| format!("reproduction: {e:?}"))?;
                self.push_mass(
                    cell_i,
                    lineage.id,
                    ReasonCode::Reproduction,
                    Pool::Nutrient,
                    Pool::Biomass,
                    gain,
                );
                self.push_energy(
                    cell_i,
                    lineage.id,
                    ReasonCode::Reproduction,
                    Pool::Biomass,
                    Pool::Nutrient,
                    gain,
                );
            }
        }
        Ok(())
    }

    fn lineage_emission(&mut self) -> Result<(), String> {
        self.sync_life_slots();
        let lineages = self.state.lineages.clone();
        for cell_i in 0..self.state.grid.cells.len() {
            for lineage in &lineages {
                let id = lineage.id as usize;
                if id >= 8 || self.life[cell_i][id] == LIFE_ABSENT {
                    continue;
                }
                let amount = self.state.grid.cells[cell_i].biomass[id]
                    .min(lineage.waste_emission.max(0));
                if amount <= 0 {
                    continue;
                }
                self.state.grid.cells[cell_i].biomass[id] -= amount;
                self.state.grid.cells[cell_i].waste = fixed::add(
                    self.state.grid.cells[cell_i].waste,
                    amount,
                )
                .map_err(|e| format!("emission: {e:?}"))?;
                self.push_mass(
                    cell_i,
                    lineage.id,
                    ReasonCode::Emission,
                    Pool::Biomass,
                    Pool::Waste,
                    amount,
                );
            }
        }
        Ok(())
    }
}
