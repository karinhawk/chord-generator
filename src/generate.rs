use std::io::Write;

use realfft::RealFftPlanner;

use crate::{note_to_frequency, wav::{AVG_BYTES_PER_SECOND, SAMPLES_PER_SECOND, create_wav_buffer}, wave::Wave};

pub fn generate(wave_type: Wave, notes: Vec<String>, duration: f64) -> Result<(), std::io::Error> {
    let length = SAMPLES_PER_SECOND as usize;

    let mut real_planner = RealFftPlanner::<f64>::new();
    let r2c = real_planner.plan_fft_inverse(length);
    let mut spectrum = r2c.make_input_vec();

    for note in notes {
        match note_to_frequency(&note) {
            Ok(freq) => {
                println!(
                    "adding {} Hz to chord",
                    freq
                );

                wave_type.create_wave(
                    &mut spectrum,
                    freq as f64,
                    600.0,
                    wave_type.harmonics(),
                );
            }

            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        }
    }

    let total_samples=
        AVG_BYTES_PER_SECOND * duration as u32;

    let mut wav_file_buffer = create_wav_buffer("audio.wav", total_samples as usize)?;

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

            wav_file_buffer.write_all(&amplitude.to_le_bytes())?;
        }
    }

    wav_file_buffer.flush()?;

    println!("generated WAV file");

    Ok(())
}