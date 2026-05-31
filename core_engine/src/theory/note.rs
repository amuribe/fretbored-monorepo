use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct MidiNote(pub u8);
// Convert scientific pitch notation (A4) into the corresponding MIDI number
impl TryFrom<String> for MidiNote {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err("Empty pitch string".to_string());
        }

        // Separate the chars of the string
        let mut chars = s.chars().peekable();
        // Empty buffer to write the note to
        let mut pitch_class_str = String::new();

        // Extract root note letter
        if let Some(first_char) = chars.next() {
            // Ensure first character is alphabetical
            if !first_char.is_ascii_alphabetic() {
                return Err(format!(
                    "Invalid starting character in pitch: {}",
                    first_char
                ));
            }
            // Uppercase so b is not confused with flat
            pitch_class_str.push(first_char.to_ascii_uppercase());
        }

        // Check accidental (# or b) without advancing iterator
        if let Some(&next_char) = chars.peek()
            && (next_char == '#' || next_char == 'b')
        {
            // Write accidental to string
            pitch_class_str.push(next_char);
            // Advance iterator
            chars.next();
        }

        // Final characters are the octave number
        let octave_str: String = chars.collect();

        if octave_str.is_empty() {
            return Err(format!("Missing octave of pitch: {}", s));
        }

        // Match pitch string to semitone offset (0-11)
        let pitch_class_val = match pitch_class_str.as_str() {
            "C" | "B#" => 0,
            "C#" | "Db" => 1,
            "D" => 2,
            "D#" | "Eb" => 3,
            "E" | "Fb" => 4,
            "F" | "E#" => 5,
            "F#" | "Gb" => 6,
            "G" => 7,
            "G#" | "Ab" => 8,
            "A" => 9,
            "A#" | "Bb" => 10,
            "B" | "Cb" => 11,
            _ => return Err(format!("Invalid note name: {}", pitch_class_str)),
        };

        // Parse octave as integer then calculate MIDI value using formula (octave + 1) * 12 + pitch_class_val
        let octave = octave_str
            .parse::<i8>()
            .map_err(|_| format!("Invalid octave format: {}", octave_str))?;

        let midi_val = (octave as i32 + 1) * 12 + pitch_class_val;

        // Ensure result is a valid MIDI value in range (0-127)
        if !(0..=127).contains(&midi_val) {
            return Err(format!("MIDI value {} is out of bounds (0-127)", midi_val));
        }

        // Return successful value
        Ok(MidiNote(midi_val as u8))
    }
}

// The 12 chromatic pitches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    // Note int value from enum
    pub const fn value(&self) -> u8 {
        match self {
            Note::C => 0,
            Note::CSharp => 1,
            Note::D => 2,
            Note::DSharp => 3,
            Note::E => 4,
            Note::F => 5,
            Note::FSharp => 6,
            Note::G => 7,
            Note::GSharp => 8,
            Note::A => 9,
            Note::ASharp => 10,
            Note::B => 11,
        }
    }
}

// Helper Functino:  get MidiNote from Note enum value and octave number
pub const fn midi_from_note(note_val: u8, octave: i8) -> MidiNote {
    MidiNote(((octave + 1) * 12 + note_val as i8) as u8)
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

    #[test]
    fn test_spn_to_midi() {
        // Standard pitches
        assert_eq!(MidiNote::try_from("C4".to_string()), Ok(MidiNote(60)));
        assert_eq!(MidiNote::try_from("A4".to_string()), Ok(MidiNote(69)));

        // Accidentals and case sensitivity
        assert_eq!(MidiNote::try_from("F#3".to_string()), Ok(MidiNote(54)));
        assert_eq!(MidiNote::try_from("db3".to_string()), Ok(MidiNote(49)));

        // Edge boundaries (min and max valid MIDI values)
        assert_eq!(MidiNote::try_from("C-1".to_string()), Ok(MidiNote(0)));
        assert_eq!(MidiNote::try_from("G9".to_string()), Ok(MidiNote(127)));
    }

    #[test]
    fn test_spn_parsing_failures() {
        assert!(MidiNote::try_from("H4".to_string()).is_err()); // Invalid pitch letter
        assert!(MidiNote::try_from("C".to_string()).is_err()); // Missing octave
        assert!(MidiNote::try_from("G10".to_string()).is_err()); // Out of MIDI range (>127)
    }
}
