//! D0 CLI skeleton. Cargo clap dependency is intentionally deferred.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("verify") if args.get(2).map(String::as_str) == Some("--suite") => {
            println!("D0 skeleton: verify suite requested");
        }
        _ => eprintln!("usage: sim-cli verify --suite week1"),
    }
}
