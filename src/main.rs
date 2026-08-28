use std::{io::Write, println, process::exit, str::FromStr};
use clap::{Parser, Subcommand, ValueEnum};
use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;
use zerocopy::{
    Immutable,
    IntoBytes,
    little_endian::{U16, U32},
};
use std::io::BufWriter;

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

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Wave {
    Sine,
    Square,
    Saw,
}

impl Wave {
    fn create_wave(
        self,
        spectrum: &mut [Complex<f64>],
        freq: u32,
        amplitude: f64,
        harmonics: u32,
    ) {
        match self {
            Self::Sine => {
                add_frequency(spectrum, freq.into(), amplitude)
            }
            Self::Saw => add_harmonics(spectrum, freq.into(), amplitude, harmonics, false),
            Self::Square => add_harmonics(spectrum, freq.into(), amplitude, harmonics, true)
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
enum Note {
    C = 261,
    Db = 277,
    D = 293,
    Eb = 311,
    E = 329,
    F = 349,
    Gb = 369,
    G = 392,
    Ab = 415,
    A = 440,
    Bb = 466,
    B = 493,
}

impl Note {
    fn freq(self) -> u32 {
        self as u32
    }

    fn octave(self, n: u32) -> u32 {
        self.freq() * (1 << n)
    }
}

impl FromStr for Note {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(Note::C),
            "Db" => Ok(Note::Db),
            "D" => Ok(Note::D),
            "Eb" => Ok(Note::Eb),
            "E" => Ok(Note::E),
            "F" => Ok(Note::F),
            "Gb" => Ok(Note::Gb),
            "G" => Ok(Note::G),
            "Ab" => Ok(Note::Ab),
            "A" => Ok(Note::A),
            "Bb" => Ok(Note::Bb),
            "B" => Ok(Note::B),
            _ => Err(format!(
                "Unknown note: {}; please use the flat enharmonic",
                s
            )),
        }
    }
}

struct MorphNote {
    start_freq: f64,
    end_freq: f64
}

#[derive(IntoBytes, Immutable)]
#[repr(u16)]
enum WaveFormatCategory {
    Pcm = 0x0001u16.to_le(),
}

#[derive(IntoBytes, Immutable)]
#[repr(C, packed)]
struct FormatChunkCommon<FSF> {
    format_tag: WaveFormatCategory,
    channels: U16,
    samples_per_sec: U32,
    avg_bytes_per_sec: U32,
    block_align: U16,
    format_specific: FSF,
}

#[derive(IntoBytes, Immutable)]
#[repr(C, packed)]
struct FormatChunkPcm {
    bits_per_sample: U16,
}

const CHANNELS: u16 = 1;
const SAMPLES_PER_SECOND: u32 = 44100;
const BITS_PER_SAMPLE: u16 = 16;

const AVG_BYTES_PER_SECOND: u32 =
    CHANNELS as u32
        * SAMPLES_PER_SECOND
        * (BITS_PER_SAMPLE / 8) as u32;

/// when we morph, we will dealing with floats basically so we need a way to normalise those
/// so they work with the type Complex is expecting
fn add_frequency(
    spectrum: &mut [Complex<f64>],
    freq: f64,
    amplitude: f64
) {
    let lower = freq.floor() as usize;

    if lower >= spectrum.len() {
        return;
    }

    let upper = lower + 1;

    let upper_amount = freq - lower as f64;
    let lower_amount = 1.0 - upper_amount;

    spectrum[lower] += Complex::from(amplitude * lower_amount);

    if upper < spectrum.len() {
        spectrum[upper] += Complex::from(amplitude * upper_amount);
    }
}

fn add_harmonics(
    spectrum: &mut [Complex<f64>],
    freq: f64,
    amplitude: f64,
    harmonics: u32,
    square: bool,
) {
    for harmonic in 1..harmonics {
        if square && harmonic % 2 == 0 {
            continue;
        }

        let harmonic_freq = freq * harmonic as f64;
        let harmonic_amplitude =
            amplitude / harmonic as f64;

        add_frequency(
            spectrum,
            harmonic_freq,
            harmonic_amplitude,
        );
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

fn morph(
    wave_type: Wave,
    from_notes: Vec<String>,
    to_notes: Vec<String>,
    duration: f64,
) -> Result<(), std::io::Error> {

    if from_notes.len() != to_notes.len() {
        eprintln!(
            "The starting and destination chords must contain \
             the same number of notes"
        );
        exit(1);
    }

    if from_notes.is_empty() {
        eprintln!("Both chords must contain at least one note");
        exit(1);
    }

    if duration <= 0.0 {
        eprintln!("Duration must be greater than 0 seconds");
        exit(1);
    }

    let notes: Vec<MorphNote> = from_notes
        .iter()
        .zip(to_notes.iter())
        .map(|(from, to)| {
            let start_freq = note_to_frequency(from)
                .map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e,
                ))?;

            let end_freq = note_to_frequency(to)
                .map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e,
                ))?;

            Ok(MorphNote {
                start_freq: start_freq as f64,
                end_freq: end_freq as f64,
            })
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    println!("Morphing:");

    for note in &notes {
        println!(
            "{} Hz → {} Hz",
            note.start_freq,
            note.end_freq
        );
    }

    let total_samples =
        (SAMPLES_PER_SECOND as f64 * duration) as usize;

    let sample_data_len =
        (total_samples * std::mem::size_of::<i16>()) as u32;

    let format = FormatChunkCommon {
        format_tag: WaveFormatCategory::Pcm,
        channels: CHANNELS.into(),
        samples_per_sec: SAMPLES_PER_SECOND.into(),
        avg_bytes_per_sec: AVG_BYTES_PER_SECOND.into(),
        block_align: (CHANNELS * BITS_PER_SAMPLE / 8).into(),
        format_specific: FormatChunkPcm {
            bits_per_sample: BITS_PER_SAMPLE.into(),
        },
    };

    let out = std::fs::File::create("audio.wav")?;
    let mut out = BufWriter::new(out);

    out.write_all(b"RIFF")?;

    out.write_all(
        &(sample_data_len
            + 3 * 4
            + std::mem::size_of_val(&format) as u32)
            .to_le_bytes(),
    )?;

    out.write_all(b"WAVE")?;

    write_chunk(b"fmt ", format, &mut out)?;

    out.write_all(b"data")?;
    out.write_all(&sample_data_len.to_le_bytes())?;

    let amplitude = 600.0;

    let harmonics = match wave_type {
        Wave::Saw => 14,
        Wave::Square => 40,
        Wave::Sine => 1,
    };

    let mut phases = vec![0.0; notes.len()];

    for sample_index in 0..total_samples {

        let progress =
            sample_index as f64 / (total_samples - 1) as f64;

        let mut sample = 0.0;

        for (note_index, note) in notes.iter().enumerate() {
            let frequency =
                note.start_freq
                    + (note.end_freq - note.start_freq) * progress;

            let phase_increment =
                frequency / SAMPLES_PER_SECOND as f64;

            let value = match wave_type {
                Wave::Sine => {
                    (phases[note_index]
                        * std::f64::consts::TAU)
                        .sin()
                }

                Wave::Square => {
                    let mut value = 0.0;

                    for harmonic in 1..=harmonics {
                        if harmonic % 2 == 0 {
                            continue;
                        }

                        let harmonic_phase =
                            phases[note_index]
                                * harmonic as f64;

                        value +=
                            harmonic_phase
                                .sin()
                                / harmonic as f64;
                    }

                    value
                }

                Wave::Saw => {
                    let mut value = 0.0;

                    for harmonic in 1..=harmonics {
                        let harmonic_phase =
                            phases[note_index]
                                * harmonic as f64;

                        value +=
                            harmonic_phase
                                .sin()
                                / harmonic as f64;
                    }

                    value
                }
            };

            sample += value * amplitude;

            phases[note_index] += phase_increment;

            if phases[note_index] >= 1.0 {
                phases[note_index] -=
                    phases[note_index].floor();
            }
        }

        let sample = sample
            .clamp(i16::MIN as f64, i16::MAX as f64)
            as i16;

        out.write_all(&sample.to_le_bytes())?;
    }

    out.flush()?;

    println!(
        "generated {} second morph WAV file",
        duration
    );

    Ok(())
}

fn generate(wave_type: Wave, notes: Vec<String>, duration: f64) -> Result<(), std::io::Error> {
    let length = SAMPLES_PER_SECOND as usize;

    let mut real_planner = RealFftPlanner::<f64>::new();
    let r2c = real_planner.plan_fft_inverse(length);
    let mut spectrum = r2c.make_input_vec();

    for note in notes {
        let (octave, note_str) =
            if let Some(stripped_note) = note.strip_prefix('u') {
                (1, stripped_note)
            } else {
                (0, note.as_str())
            };

        match note_str.parse::<Note>() {
            Ok(note) => {
                let freq = note.octave(octave);
                let amplitude = 600.0;

                let harmonics = match wave_type {
                    Wave::Saw => 14,
                    Wave::Square => 40,
                    Wave::Sine => 1,
                };

                println!(
                    "adding {:?} ({} Hz) to chord",
                    note, freq
                );

                wave_type.create_wave(
                    &mut spectrum,
                    freq,
                    amplitude,
                    harmonics,
                );
            }

            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        }
    }

    let sample_data_len=
        AVG_BYTES_PER_SECOND * duration as u32;

    let format = FormatChunkCommon {
        format_tag: WaveFormatCategory::Pcm,
        channels: 1.into(),
        samples_per_sec: SAMPLES_PER_SECOND.into(),
        avg_bytes_per_sec: AVG_BYTES_PER_SECOND.into(),
        // channels * bits per sample / 8
        block_align: (CHANNELS * BITS_PER_SAMPLE / 8).into(),
        format_specific: FormatChunkPcm {
            bits_per_sample: BITS_PER_SAMPLE.into(),
        },
    };

    let out = std::fs::File::create("audio.wav")?;
    let mut out = BufWriter::new(out);

    out.write_all(b"RIFF")?;

    out.write_all(
        &(sample_data_len
            + 3 * 4
            + std::mem::size_of_val(&format) as u32)
            .to_le_bytes(),
    )?;

    out.write_all(b"WAVE")?;

    write_chunk(b"fmt ", format, &mut out)?;

    out.write_all(b"data")?;
    // format specific for PCM:
    // WORD wBitsPerSample
    out.write_all(&sample_data_len.to_le_bytes())?;

    let mut time = r2c.make_output_vec();

    r2c.process(&mut spectrum, &mut time).unwrap();

    let mut dampen = -1.0;

    for _interval in 0..duration as u32 {
        for sample in &time {
            let amplitude = sample.round();
            let amplitude = amplitude + amplitude * dampen;

            let amplitude = (amplitude as i64)
                .clamp(i16::MIN as i64, i16::MAX as i64)
                as i16;

            dampen = (dampen + 0.0001).min(0.0);

            out.write_all(&amplitude.to_le_bytes())?;
        }
    }

    out.flush()?;

    println!("generated WAV file");

    Ok(())
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

fn write_chunk<T: IntoBytes + Immutable, W: Write>(
    fourcc: &[u8; 4],
    t: T,
    mut out: W,
) -> Result<(), std::io::Error> {
    out.write_all(fourcc)?;
    out.write_all(
        &(std::mem::size_of::<T>() as u32).to_le_bytes()
    )?;
    t.write_to_io(&mut out)?;
    Ok(())
}
