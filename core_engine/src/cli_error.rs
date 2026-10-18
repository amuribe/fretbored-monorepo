use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    UnknownTuning(String),
    UnknownScale(String),
    InvalidNote(String),
    InvalidChord(String),
    InvalidFret { fret: u8, max: u8 },
    InvalidString { index: usize, count: usize },
    NoVoicings,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownTuning(name) => write!(f, "unknown tuning '{name}'"),
            CliError::UnknownScale(name) => write!(f, "unknown scale '{name}'"),
            CliError::InvalidNote(input) => write!(f, "invalid note '{input}'"),
            CliError::InvalidChord(input) => write!(f, "invalid chord '{input}'"),
            CliError::InvalidFret { fret, max } => {
                write!(f, "fret {fret} is out of range (0-{max})")
            }
            CliError::InvalidString { index, count } => {
                write!(f, "string {index} is out of range (0-{})", count - 1)
            }
            CliError::NoVoicings => write!(f, "No voicings found"),
        }
    }
}

impl Error for CliError {}
