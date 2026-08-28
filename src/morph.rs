use std::{io::Write, process::exit};

use crate::{note::MorphNote, note_to_frequency, wav::{SAMPLES_PER_SECOND, create_wav_buffer}, wave::Wave};

pub fn morph(
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
            "{} Hz -> {} Hz",
            note.start_freq,
            note.end_freq
        );
    }

    let total_samples =
        (SAMPLES_PER_SECOND as f64 * duration) as usize;
    let mut wav_buffer = create_wav_buffer("audio.wav", total_samples)?;

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

            let value = wave_type.sample(phases[note_index], harmonics);

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

        wav_buffer.write_all(&sample.to_le_bytes())?;
    }

    wav_buffer.flush()?;

    println!(
        "generated {} second morph WAV file",
        duration
    );

    Ok(())
}