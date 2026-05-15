// The 12 chromatic pitches
#[derive(Debug, PartialEq, Eq)]
pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Note {
    // Convert an abs MIDI note number to a pitch
    // Modulo wraps so note must be 0-11
    pub fn from_midi(midi_val: u8) -> Self {
        match midi_val % 12 {
            0 => Note::C,
            1 => Note::CSharp,
            2 => Note::D,
            3 => Note::DSharp,
            4 => Note::E,
            5 => Note::F,
            6 => Note::FSharp,
            7 => Note::G,
            8 => Note::GSharp,
            9 => Note::A,
            10 => Note::ASharp,
            11 => Note::B,
            _ => unreachable!("Modulo 12 ensures values are 0-11"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_note() {
        assert_eq!(Note::from_midi(60), Note::C); // Middle C
        assert_eq!(Note::from_midi(61), Note::CSharp); // C#
        assert_eq!(Note::from_midi(40), Note::E); // Low E guitar (6th-string standard tuning)
    }
}
