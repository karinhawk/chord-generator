use clap::ValueEnum;
use rustfft::num_complex::Complex;

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Wave {
    Sine,
    Square,
    Saw,
}

impl Wave {
    pub fn create_wave(
        self,
        spectrum: &mut [Complex<f64>],
        freq: f64,
        amplitude: f64,
        harmonics: u32,
    ) {
        match self {
            Self::Sine => {
                Self::add_frequency(spectrum, freq.into(), amplitude)
            }
            Self::Saw => Self::add_harmonics(spectrum, freq.into(), amplitude, harmonics, false),
            Self::Square => Self::add_harmonics(spectrum, freq.into(), amplitude, harmonics, true)
        }
    }

    pub fn harmonics(self) -> u32 {
        match self {
            Wave::Sine => 1,
            Wave::Square => 40,
            Wave::Saw => 14,
        }
    }

    pub fn sample(
        self,
        phase: f64,
        harmonics: u32,
    ) -> f64 {
        match self {
            Wave::Sine => {
                (phase * std::f64::consts::TAU).sin()
            }

            Wave::Square => {
                (phase * std::f64::consts::TAU)
                    .sin()
                    .signum()
            }

            Wave::Saw => {
                let mut value = 0.0;

                for harmonic in 1..=harmonics {
                    let harmonic_phase =
                        phase * harmonic as f64;

                    value += harmonic_phase.sin()
                        / harmonic as f64;
                }

                value
            }
        }}

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

                Self::add_frequency(
                    spectrum,
                    harmonic_freq,
                    harmonic_amplitude,
                );
            }
        }
}