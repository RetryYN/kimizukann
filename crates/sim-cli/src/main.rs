use kimizukann_sim_core::{fixed, SimCore};
use kimizukann_sim_types::{LineageParams, MechanismTags, StateHash, TraitVector, FIXED_SCALE};
use sha2::{Digest, Sha256};

const WEEK1_GOLDEN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/30_contracts/golden/d1-week1.json"
));

fn lineage(id: u8) -> LineageParams {
    LineageParams {
        id,
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

fn hex_hash(hash: StateHash) -> String {
    hash.0.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Independent normalization oracle used by gate-selftest to catch hash drift.
fn reference_hash(sim: &SimCore) -> StateHash {
    let mut h = Sha256::new();
    h.update(sim.state.tick.to_le_bytes());
    h.update(sim.seed.0.to_le_bytes());
    h.update(sim.state.grid.width.to_le_bytes());
    h.update(sim.state.grid.height.to_le_bytes());
    h.update(sim.model_version.as_bytes());
    for stream in &sim.rng {
        for word in stream.words() {
            h.update(word.to_le_bytes());
        }
    }
    for cell in &sim.state.grid.cells {
        h.update(cell.nutrient.to_le_bytes());
        for value in cell.biomass {
            h.update(value.to_le_bytes());
        }
        h.update(cell.carcass.to_le_bytes());
        h.update(cell.waste.to_le_bytes());
        for value in cell.energy {
            h.update(value.to_le_bytes());
        }
        h.update(cell.occupancy_peak.to_le_bytes());
    }
    StateHash(h.finalize().into())
}

fn one_tick_reference() -> bool {
    let mut sim = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage(0)]);
    if sim.step(1).is_err() {
        return false;
    }
    if fixed::split_output(3, 500_000) != Ok((2, 1)) {
        return false;
    }
    let cell = &sim.state.grid.cells[0];
    cell.nutrient == 9_630_000
        && cell.biomass[0] == 2_339_999
        && cell.waste == 30_001
        && cell.energy[0] == 290_000
}

fn week1_golden_matches(suite: &str, state_hash: &str) -> bool {
    if !matches!(suite, "week1" | "all") {
        return true;
    }
    let golden: serde_json::Value = match serde_json::from_str(WEEK1_GOLDEN) {
        Ok(value) => value,
        Err(_) => return false,
    };
    golden["schema_version"] == "golden-v1"
        && golden["case"] == "d1-one-cell-week1"
        && golden["model_version"] == "d1-v1;prng=xoshiro256ss-v1;hash=sha256-v1"
        && golden["config"]["seed"] == 7
        && golden["config"]["grid"] == "one-cell"
        && golden["config"]["nutrient"] == 10_000_000
        && golden["config"]["biomass"] == 2_000_000
        && golden["config"]["lineage_preset"] == "d1-lineage-0"
        && golden["config"]["ticks"] == 2_000
        && golden["seed"] == 7
        && golden["ticks"] == 2_000
        && golden["state_hash"].as_str() == Some(state_hash)
}

fn verify_suite(suite: &str) -> (bool, bool, bool, bool, bool, bool, String) {
    let ticks = match suite {
        "quick" => 20,
        "week1" | "all" => 2_000,
        _ => return (false, false, false, false, false, false, String::new()),
    };
    let mut a = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage(0)]);
    let mut b = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage(0)]);
    let conservation = a.step(ticks).is_ok() && a.invariant_report().mass_ok;
    let determinism = b.step(ticks / 2).is_ok()
        && b.step(ticks - ticks / 2).is_ok()
        && a.state_hash() == b.state_hash();
    let nonneg = a.invariant_report().non_negative;
    let transition = one_tick_reference();
    let hash_normalization = a.state_hash() == reference_hash(&a);
    let state_hash = hex_hash(a.state_hash());
    let golden = week1_golden_matches(suite, &state_hash);
    (
        conservation,
        determinism,
        nonneg,
        transition,
        hash_normalization,
        golden,
        state_hash,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("verify")
        || args.get(2).map(String::as_str) != Some("--suite")
    {
        eprintln!("usage: sim-cli verify --suite quick|week1|all|D2");
        std::process::exit(2);
    }
    let suite = args.get(3).map(String::as_str).unwrap_or_default();
    if suite == "D2" {
        let (conservation, symmetry) = SimCore::verify_suite_d2();
        let ok = conservation && symmetry;
        println!(
            "{{\"suite\":\"D2\",\"conservation_64x64\":{conservation},\"symmetry\":{symmetry},\"status\":\"{}\"}}",
            if ok { "pass" } else { "fail" }
        );
        std::process::exit(if ok { 0 } else { 1 });
    }
    if !matches!(suite, "quick" | "week1" | "all") {
        eprintln!("usage: sim-cli verify --suite quick|week1|all|D2");
        std::process::exit(2);
    }
    let (conservation, determinism, nonneg, transition, hash_normalization, golden, state_hash) =
        verify_suite(suite);
    let ok = conservation && determinism && nonneg && transition && hash_normalization && golden;
    println!(
        "{{\"suite\":\"{suite}\",\"conservation_1cell\":{conservation},\"determinism\":{determinism},\"nonneg\":{nonneg},\"transition_reference\":{transition},\"hash_normalization\":{hash_normalization},\"golden\":{golden},\"state_hash\":\"{state_hash}\",\"state_hashes\":[\"{state_hash}\"],\"status\":\"{}\"}}",
        if ok { "pass" } else { "fail" }
    );
    if !ok {
        std::process::exit(1);
    }
}
