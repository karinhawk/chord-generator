use std::{env, io::Write, str::FromStr};

use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;
use zerocopy::{
    Immutable,
    IntoBytes,
    little_endian::{U16, U32},
};
use std::io::BufWriter;

#[derive(Debug, Copy, Clone)]
enum Wave {
    SINE,
    SQUARE,
    SAW,
}

impl Wave {
    fn create_wave(
        self,
        spectrum: Vec<Complex<f64>>,
        freq: u32,
        amplitude: f64,
        harmonics: u32,
    ) -> Vec<Complex<f64>> {
        match self {
            Self::SAW => create_sawtooth_frequencies(
                spectrum,
                freq,
                amplitude,
                harmonics,
            ),
            Self::SINE => {
                create_sine_frequencies(spectrum, freq, amplitude)
            }
            Self::SQUARE => create_square_frequencies(
                spectrum,
                freq,
                amplitude,
                harmonics,
            ),
        }
    }
}

impl FromStr for Wave {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sine" => Ok(Wave::SINE),
            "square" => Ok(Wave::SQUARE),
            "saw" => Ok(Wave::SAW),
            _ => Err(format!(
                "invalid wave type: {} (use sine, square, or saw)",
                s
            )),
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
                "Unknown note, please use the flat enharmonic: {}",
                s
            )),
        }
    }
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

fn create_sine_frequencies(
    mut spectrum: Vec<Complex<f64>>,
    freq: u32,
    amplitude: f64,
) -> Vec<Complex<f64>> {
    spectrum[freq as usize] = amplitude.into();
    spectrum
}

fn create_sawtooth_frequencies(
    mut spectrum: Vec<Complex<f64>>,
    freq: u32,
    amplitude: f64,
    harmonics: u32,
) -> Vec<Complex<f64>> {
    for harmonic in 1..harmonics {
        let freq = freq * harmonic;
        let amplitude = amplitude * (1.0 / harmonic as f64);

        spectrum[freq as usize] =
            Complex::<f64>::from(amplitude);
    }

    spectrum
}

fn create_square_frequencies(
    mut spectrum: Vec<Complex<f64>>,
    freq: u32,
    amplitude: f64,
    harmonics: u32,
) -> Vec<Complex<f64>> {
    for harmonic in 1..harmonics {
        if harmonic % 2 == 0 {
            continue;
        }

        let freq = freq * harmonic;
        let amplitude = amplitude * (1.0 / harmonic as f64);

        spectrum[freq as usize] =
            Complex::<f64>::from(amplitude);
    }

    spectrum
}

fn generate(wave_type: Wave, notes: Vec<String>) -> Result<(), std::io::Error> {
    let length = SAMPLES_PER_SECOND as usize;

    let mut real_planner = RealFftPlanner::<f64>::new();
    let r2c = real_planner.plan_fft_inverse(length);
    let mut spectrum = r2c.make_input_vec();

    for note in notes {
        let (octave, note_str) =
            if let Some(rest) = note.strip_prefix('u') {
                (1, rest)
            } else {
                (0, note.as_str())
            };

        match note_str.parse::<Note>() {
            Ok(note) => {
                let freq = note.octave(octave);
                let amplitude = 600.0;

                let harmonics = match wave_type {
                    Wave::SAW => 14,
                    Wave::SQUARE => 40,
                    Wave::SINE => 1,
                };

                println!(
                    "adding {:?} ({} Hz) to chord",
                    note, freq
                );

                spectrum = wave_type.create_wave(
                    spectrum,
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

    let duration_in_seconds = 10;
    let sample_data_len =
        AVG_BYTES_PER_SECOND * duration_in_seconds;

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

    for _interval in 0..duration_in_seconds {
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

fn print_usage() {
    eprintln!(
        r#"Usage:
    chord-generator generate <wave> <notes...>

Examples:
    chord-generator generate sine C E G
    chord-generator generate square C E G
    chord-generator generate saw C E G
    chord-generator generate sine C uE G

Wave types:
    sine
    square
    saw

Notes:
    C Db D Eb E F Gb G Ab A Bb B

Prefix a note with 'u' to play it one octave higher.
"#
    );
}

fn main() -> Result<(), std::io::Error> {
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    match command.as_str() {
        "generate" => {
            let Some(wave_type) = args.next() else {
                eprintln!("missing wave type");
                print_usage();
                std::process::exit(1);
            };

            let wave_type: Wave = match wave_type.parse() {
                Ok(wave) => wave,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            let notes: Vec<String> = args.collect();

            if notes.is_empty() {
                eprintln!("missing notes");
                print_usage();
                std::process::exit(1);
            }

            generate(wave_type, notes)?;
        }

        "help" | "--help" | "-h" => {
            print_usage();
        }

        unknown => {
            eprintln!("unknown command: {}", unknown);
            print_usage();
            std::process::exit(1);
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
