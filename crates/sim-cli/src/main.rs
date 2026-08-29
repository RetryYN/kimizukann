use kimizukann_sim_core::SimCore;
use kimizukann_sim_types::{LineageParams, MechanismTags, TraitVector, FIXED_SCALE};

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("verify")
        || args.get(2).map(String::as_str) != Some("--suite")
        || args.get(3).is_none()
    {
        eprintln!("usage: sim-cli verify --suite week1|D2");
        std::process::exit(2);
    }
    let suite = args[3].as_str();
    if suite == "D2" || suite == "d2" {
        let (conservation, symmetry) = SimCore::verify_suite_d2();
        let ok = conservation && symmetry;
        println!(
            "{{\"suite\":\"D2\",\"conservation_64x64\":{},\"symmetry\":{},\"status\":\"{}\"}}",
            conservation,
            symmetry,
            if ok { "pass" } else { "fail" }
        );
        std::process::exit(if ok { 0 } else { 1 });
    }
    if suite != "week1" {
        eprintln!("usage: sim-cli verify --suite week1|D2");
        std::process::exit(2);
    }
    let mut a = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage(0)]);
    let mut b = SimCore::one_cell(7, 10 * FIXED_SCALE, 2 * FIXED_SCALE, vec![lineage(0)]);
    let conservation = { a.step(2000).is_ok() && a.invariant_report().mass_ok };
    let determinism =
        { b.step(1000).is_ok() && b.step(1000).is_ok() && a.state_hash() == b.state_hash() };
    let nonneg = a.invariant_report().non_negative;
    let ok = conservation && determinism && nonneg;
    println!("{{\"suite\":\"week1\",\"conservation_1cell\":{},\"determinism\":{},\"nonneg\":{},\"status\":\"{}\"}}", conservation, determinism, nonneg, if ok { "pass" } else { "fail" });
    if !ok {
        std::process::exit(1);
    }
}
