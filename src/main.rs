use agent_exporter::cli;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("agent-exporter failed: {error:#}");
        std::process::exit(1);
    }
}
