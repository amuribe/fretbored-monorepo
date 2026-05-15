use crate::theory::Note;

// The notes of each string on the instrument and the corresponding midi numbers
pub struct Tuning {
    pub open_strings: Vec<u8>,
}

impl Tuning {
    // Associated function (constructor) to easily instantiate a common tuning.
    pub fn standard_guitar() -> Self {
        // Construct a 6 string tuning system {E, A, D, G, B, E}
        Self {
            open_strings: vec![40, 45, 50, 55, 59, 64], // Low E to high E
        }
    }

    // Get the note produce by pressing a specified fret on a specific string
    pub fn note_at_fret(&self, string_index: usize, fret: u8) -> Option<Note> {
        // '?' returns null if out of bounds
        let open_midi = self.open_strings.get(string_index)?;

        let fretted_midi = open_midi + fret;

        // Get note from midi value of fretted note
        Some(Note::from_midi(fretted_midi))
    }
}

// Test Tuning
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_at_fret() {
        // Create a guitar tuning system
        let guitar = Tuning::standard_guitar();

        // String 0 (Low E, MIDI 40) + Fret 3 = MIDI 43 (Note::G)
        assert_eq!(guitar.note_at_fret(0, 3), Some(Note::G));

        // String 5 (High E, MIDI 64) + Fret 0 = MIDI 64 (Note::E)
        assert_eq!(guitar.note_at_fret(5, 0), Some(Note::E));

        // Out of bounds string index should return None
        assert_eq!(guitar.note_at_fret(99, 0), None);
    }
}
