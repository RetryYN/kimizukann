use std::process::Command;

fn run(suite: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_kimizukann-sim-cli"))
        .args(["verify", "--suite", suite])
        .output()
        .expect("sim-cli must start");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).expect("verify output must be JSON")
}

#[test]
fn quick_and_all_are_supported_and_emit_state_hash() {
    for suite in ["quick", "all"] {
        let report = run(suite);
        assert_eq!(report["suite"], suite);
        assert_eq!(report["status"], "pass");
        assert!(report["state_hash"].as_str().is_some_and(|value| value.len() == 64));
    }
}

#[test]
fn unknown_suite_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_kimizukann-sim-cli"))
        .args(["verify", "--suite", "unknown"])
        .output()
        .expect("sim-cli must start");
    assert_eq!(output.status.code(), Some(2));
}
