use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum Note {
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
    pub fn freq(self) -> u32 {
        self as u32
    }

    pub fn octave(self, n: u32) -> u32 {
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

pub struct MorphNote {
    pub start_freq: f64,
    pub end_freq: f64
}