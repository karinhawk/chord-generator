mod morph;
mod generate;
mod wave;
mod note;
mod wav;

use clap::{Parser, Subcommand};

use crate::{note::Note, wave::Wave, morph::morph, generate::generate};

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

fn note_to_frequency(note: &str) -> Result<u32, String> {
    let (octave, note_str) =
        if let Some(stripped_note) = note.strip_prefix('u') {
            (1, stripped_note)
        } else {
            (0, note)
        };

    let note = note_str.parse::<Note>()?;

    Ok(note.octave(octave))
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
