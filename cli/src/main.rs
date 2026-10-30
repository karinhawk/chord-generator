use chord_generator::wave::Wave;
use chord_generator::generate::generate;
use chord_generator::morph::morph;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "chords")]
#[command(about = "Generate and manipulate chords")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate a chord
    Generate {
        /// Waveform used to generate
        #[arg(value_enum)]
        wave: Wave,

        /// Notes to include in the chord
        #[arg(required=true)]
        notes: Vec<String>,

        /// Duration of the chord in seconds
        #[arg(short, long)]
        duration: f64,
    },

    Morph {
        #[arg(value_enum)]
        wave: Wave,

        /// Notes in the starting chord
        #[arg(required = true)]
        from: Vec<String>,

        /// Notes in the destination chord
        #[arg(long, required = true, num_args = 1..)]
        to: Vec<String>,

        /// Duration of the morph in seconds
        #[arg(short, long)]
        duration: f64,
    }
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { wave, notes, duration } => {
            generate(wave, notes, duration)?;
        },
        Commands::Morph { wave, from, to, duration } => {
            morph(wave, from, to, duration)?;
        }
    }

    Ok(())
}
