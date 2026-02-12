use jetson_line_simulator::simulation::{SiteState, Lattice, Params, PainMetrics, FearRegime};
use jetson_line_simulator::ledger::Deed;
use clap::{Parser, Subcommand};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "jetson_line_sim")]
#[command(about = "CLI for Jetson_Line Simulation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SimulateEpisodes {
        episodes: usize,
        lattice_size: usize,
    },
    ComputePain {
        episode_file: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::SimulateEpisodes { episodes, lattice_size } => {
            let params = Params::default();
            for e in 0..episodes {
                let mut lattice = Lattice::new(lattice_size, &params);
                for _ in 0..100 {  // Simulate 100 ticks per episode
                    lattice.step();
                }
                let json = serde_json::to_string(&lattice).unwrap();
                let mut file = File::create(format!("episode_{}.json", e))?;
                file.write_all(json.as_bytes())?;
                println!("Episode {} simulated and saved.", e);
            }
        }
        Commands::ComputePain { episode_file } => {
            let json = std::fs::read_to_string(episode_file)?;
            let lattice: Lattice = serde_json::from_str(&json)?;
            let pain = PainMetrics::compute_from_lattice(&lattice);
            println!("Pain Metrics: {:?}", pain);
            if pain.is_teachable() {
                println!("Suggesting eco_grant for repair: {}", pain.repair_necessity * 2.0);
            }
        }
    }
    Ok(())
}
