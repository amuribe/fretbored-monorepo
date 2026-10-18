use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
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

// Error type for parsing failures
#[derive(Debug, PartialEq, Eq)]
pub struct ParseNoteError(pub String);

impl fmt::Display for ParseNoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse note: '{}'", self.0)
    }
}

impl std::error::Error for ParseNoteError {}
impl FromStr for Note {
    type Err = ParseNoteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().as_bytes() {
            [b'C' | b'c'] => Ok(Note::C),
            [b'C' | b'c', b'#'] | [b'D' | b'd', b'b' | b'B'] => Ok(Note::CSharp),
            [b'D' | b'd'] => Ok(Note::D),
            [b'D' | b'd', b'#'] | [b'E' | b'e', b'b' | b'B'] => Ok(Note::DSharp),
            [b'E' | b'e'] => Ok(Note::E),
            [b'F' | b'f'] => Ok(Note::F),
            [b'F' | b'f', b'#'] | [b'G' | b'g', b'b' | b'B'] => Ok(Note::FSharp),
            [b'G' | b'g'] => Ok(Note::G),
            [b'G' | b'g', b'#'] | [b'A' | b'a', b'b' | b'B'] => Ok(Note::GSharp),
            [b'A' | b'a'] => Ok(Note::A),
            [b'A' | b'a', b'#'] | [b'B' | b'b', b'b' | b'B'] => Ok(Note::ASharp),
            [b'B' | b'b'] => Ok(Note::B),
            _ => Err(ParseNoteError(s.to_string())),
        }
    }
}

// For handling enharmonic equivalents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
    DoubleSharp,
    DoubleFlat,
}

impl Accidental {
    pub fn symbol(&self) -> &'static str {
        match self {
            Accidental::Natural => "",
            Accidental::Sharp => "#",
            Accidental::Flat => "b",
            Accidental::DoubleSharp => "##",
            Accidental::DoubleFlat => "bb",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteSpelling {
    // The actual pitch of the note
    pub pitch: Note,
    pub letter: char,
    pub accidental: Accidental,
}

impl NoteSpelling {
    // What engine uses b default
    pub fn prefer_sharp(pitch: Note) -> Self {
        let (letter, accidental) = match pitch {
            // Naturals only have one common spelling
            Note::C => ('C', Accidental::Natural),
            Note::D => ('D', Accidental::Natural),
            Note::E => ('E', Accidental::Natural),
            Note::F => ('F', Accidental::Natural),
            Note::G => ('G', Accidental::Natural),
            Note::A => ('A', Accidental::Natural),
            Note::B => ('B', Accidental::Natural),

            // Accidentals with sharp spelling preferred
            Note::CSharp => ('C', Accidental::Sharp),
            Note::DSharp => ('D', Accidental::Sharp),
            Note::FSharp => ('F', Accidental::Sharp),
            Note::GSharp => ('G', Accidental::Sharp),
            Note::ASharp => ('A', Accidental::Sharp),
        };
        Self {
            pitch,
            letter,
            accidental,
        }
    }

    // Primary flat spelling (flat from sharp)

    pub fn prefer_flat(pitch: Note) -> Self {
        let (letter, accidental) = match pitch {
            // Accidentals with flat spelling preferred
            Note::CSharp => ('D', Accidental::Flat),
            Note::DSharp => ('E', Accidental::Flat),
            Note::FSharp => ('G', Accidental::Flat),
            Note::GSharp => ('A', Accidental::Flat),
            Note::ASharp => ('B', Accidental::Flat),

            // Naturals fall back to sharp spelling which matches to natural
            _ => return Self::prefer_sharp(pitch),
        };
        Self {
            pitch,
            letter,
            accidental,
        }
    }

    // For C = B#, E = Fb
    pub fn enharmonic_alt(pitch: Note) -> Option<Self> {
        let (letter, accidental) = match pitch {
            // C = B#
            Note::C => ('B', Accidental::Sharp),
            // E = Fb
            Note::E => ('F', Accidental::Flat),
            // F = E#
            Note::F => ('E', Accidental::Sharp),
            // B = Cb
            Note::B => ('C', Accidental::Flat),
            // No single accidental enharmonics exist for any other notes
            // May add DoubleSharp and DoubleFlat later
            _ => return None,
        };
        Some(Self {
            pitch,
            letter,
            accidental,
        })
    }

    // All valid spellings for a pitch, in order: sharp, flat, enharmonic alt
    pub fn all_spellings(pitch: Note) -> Vec<Self> {
        let mut spellings = vec![Self::prefer_sharp(pitch)];

        // Add flat spelling if it differs from sharp
        let flat = Self::prefer_flat(pitch);
        if flat != spellings[0] {
            spellings.push(flat);
        }

        // Add enharmonic alt if one exists
        if let Some(alt) = Self::enharmonic_alt(pitch) {
            spellings.push(alt);
        }

        spellings
    }

    // Display the note {letter}{accidental} -> C#
    pub fn display(&self) -> String {
        format!("{}{}", self.letter, self.accidental.symbol())
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

    // Test note spellings
    #[test]
    fn test_enharmonic_alts() {
        // C = B#
        let b_sharp = NoteSpelling::enharmonic_alt(Note::C).unwrap();
        assert_eq!(b_sharp.display(), "B#");
        assert_eq!(b_sharp.pitch, Note::C); // same pitch class

        // B = Cb
        let c_flat = NoteSpelling::enharmonic_alt(Note::B).unwrap();
        assert_eq!(c_flat.display(), "Cb");
        assert_eq!(c_flat.pitch, Note::B);

        // E = Fb
        let f_flat = NoteSpelling::enharmonic_alt(Note::E).unwrap();
        assert_eq!(f_flat.display(), "Fb");

        // F = E#
        let e_sharp = NoteSpelling::enharmonic_alt(Note::F).unwrap();
        assert_eq!(e_sharp.display(), "E#");
    }

    #[test]
    fn test_natural_notes_have_no_flat_alt() {
        // D, G, A have no common flat enharmonic — should fall back to natural
        assert_eq!(NoteSpelling::prefer_flat(Note::D).display(), "D");
        assert_eq!(NoteSpelling::prefer_flat(Note::G).display(), "G");
        assert_eq!(NoteSpelling::prefer_flat(Note::A).display(), "A");
    }

    #[test]
    fn test_all_spellings_c() {
        let spellings = NoteSpelling::all_spellings(Note::C);
        let names: Vec<String> = spellings.iter().map(|s| s.display()).collect();
        assert_eq!(names, vec!["C", "B#"]);
    }

    #[test]
    fn test_all_spellings_csharp() {
        let spellings = NoteSpelling::all_spellings(Note::CSharp);
        let names: Vec<String> = spellings.iter().map(|s| s.display()).collect();
        assert_eq!(names, vec!["C#", "Db"]);
    }

    #[test]
    fn test_pitch_class_preserved() {
        // Enharmonic spellings must never change the underlying pitch
        for spelling in NoteSpelling::all_spellings(Note::ASharp) {
            assert_eq!(spelling.pitch, Note::ASharp);
        }
    }

    // Parse tests
    #[test]
    fn test_note_from_str_valid() {
        assert_eq!("C".parse::<Note>(), Ok(Note::C));
        assert_eq!("c#".parse::<Note>(), Ok(Note::CSharp));
        assert_eq!("Db".parse::<Note>(), Ok(Note::CSharp));
        assert_eq!("  g#  ".parse::<Note>(), Ok(Note::GSharp));
    }

    #[test]
    fn test_note_from_str_invalid() {
        assert!("H".parse::<Note>().is_err());
        assert!("C##".parse::<Note>().is_err());
        assert!("".parse::<Note>().is_err());
    }
}
