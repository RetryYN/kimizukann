//! D0 contract types. Values use fixed-point mass units (scale 1_000_000).

pub const FIXED_SCALE: i64 = 1_000_000;
pub type Fixed = i64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pool { Nutrient, Biomass, Carcass, Waste }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasonCode { Intake, Maintenance, Starvation, Death, Reproduction, Emission, Diffusion }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TickPhase { Diffuse, Intake, Maintenance, Starvation, Reproduction, Emission }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminationLabel { Extinct, Fixed, Coexist, Reversal, TimeLimit }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct MechanismTags { pub use_nutrient: bool, pub use_carcass: bool, pub use_waste: bool, pub toxin_sensitive: bool, pub density_bonus: bool }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraitVector { pub movement: Fixed, pub intake: Fixed, pub conversion: Fixed, pub maintenance_cost: Fixed, pub reproduction: Fixed }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineageParams { pub id: u8, pub traits: TraitVector, pub tags: MechanismTags, pub mortality_threshold: Fixed, pub waste_emission: Fixed }

#[derive(Clone, Debug, PartialEq)]
pub struct CellState { pub nutrient: Fixed, pub biomass: [Fixed; 8], pub carcass: Fixed, pub waste: Fixed, pub energy: [Fixed; 8], pub occupancy_peak: Fixed }

#[derive(Clone, Debug, PartialEq)]
pub struct GridState { pub width: u16, pub height: u16, pub cells: Vec<CellState> }

#[derive(Clone, Debug, PartialEq)]
pub struct WorldState { pub tick: u32, pub grid: GridState, pub lineages: Vec<LineageParams> }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Seed(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrngState { pub seed: Seed, pub movement: u64, pub reproduction: u64, pub mutation: u64, pub interaction: u64 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateHash(pub [u8; 32]);

#[derive(Clone, Debug, PartialEq)]
pub struct Thresholds { pub epsilon: Fixed, pub fixed_share: Fixed, pub fixed_ticks: u32, pub coexist_share: Fixed, pub max_ticks: u32 }

#[derive(Clone, Debug, PartialEq)]
pub struct SaveEnvelope { pub schema_version: String, pub model_version: String, pub config_hash: String, pub seed: Seed, pub prng: PrngState, pub state_hash: StateHash, pub state: WorldState }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MassCoefficients { pub intake_to_biomass: Fixed, pub intake_to_waste: Fixed, pub starvation_to_carcass: Fixed, pub death_to_carcass: Fixed }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedgerEntry { pub from_pool: Pool, pub to_pool: Pool, pub amount: Fixed, pub reason: ReasonCode }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MassLedger { pub entries: Vec<LedgerEntry> }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnergyLedger { pub entries: Vec<LedgerEntry> }

#[derive(Clone, Debug, PartialEq)]
pub struct InvariantReport { pub mass_ok: bool, pub energy_ok: bool, pub non_negative: bool, pub message: String }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoundingMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumericError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifySuite;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelVersion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanOrder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RandomStream;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Substrate { Nutrient, Carcass, Waste }
