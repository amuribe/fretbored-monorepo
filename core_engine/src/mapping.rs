use crate::instrument::FretBoardConfig;
use crate::theory::Note;
use serde::{Deserialize, Serialize};

// Represents a physical coordinate on an instrument (e.g. high e string, fret 5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct NoteCoordinate {
    pub string_index: u8,
    pub fret: u8,
}

// Scans the fretboard to find all physical locations of a given set of notes.
pub fn find_notes_on_fretboard(notes: &[Note], config: &FretBoardConfig) -> Vec<NoteCoordinate> {
    // Max capacity is every fret on every string contains one of the given notes (+ 1 for open string)
    let max_possible_matches = (config.fret_count as usize + 1) * config.tuning.strings.len();
    // Allocate memory once to have the max capacity, to avoid the cost of reallocation in the search loops
    let mut coordinates = Vec::with_capacity(max_possible_matches);

    // Use a bitmask
    let mut target_mask: u16 = 0;

    for note in notes {
        // Get integer value of note
        let val = note.value();
        // After loop target_mask will have all notes on fretboard
        // mask e.g note = D -> val = 2 -> mask = 0000000000000100
        target_mask |= 1 << val;
    }

    // Go overs strings
    for (string_idx, open_midi) in config.tuning.strings.iter().enumerate() {
        // Go over fret
        for cur_fret in 0..=config.fret_count {
            // Get value of pitch
            let current_pitch = (open_midi.0 + cur_fret) % 12;

            // Check the bitmask
            // Bitwise AND checks if the bit at current_pitch is 1.
            // If the result != 0, the note appears, if == 0 the note does not
            if (target_mask & (1 << current_pitch)) != 0 {
                coordinates.push(NoteCoordinate {
                    string_index: string_idx as u8,
                    fret: cur_fret,
                });
            }
        }
    }

    // free unused capacity
    coordinates.shrink_to_fit();
    coordinates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument::{Orientation, StandardTunings};
    use crate::theory::Note;

    #[test]
    fn test_find_c_major_triad_on_standard_guitar() {
        let config = FretBoardConfig::new(
            StandardTunings::guitar_standard(),
            22,
            Orientation::RightHanded,
        );
        // C Major Triad: C(0), E(4), G(7)
        let c_major_triad = vec![Note::C, Note::E, Note::G];

        let coords = find_notes_on_fretboard(&c_major_triad, &config);

        // Low E string (index 5)
        assert!(coords.contains(&NoteCoordinate {
            string_index: 5,
            fret: 0
        })); // Open E
        assert!(coords.contains(&NoteCoordinate {
            string_index: 5,
            fret: 3
        })); // G
        assert!(coords.contains(&NoteCoordinate {
            string_index: 5,
            fret: 8
        })); // C

        // A string (index 4)
        assert!(coords.contains(&NoteCoordinate {
            string_index: 4,
            fret: 3
        })); // C
        assert!(coords.contains(&NoteCoordinate {
            string_index: 4,
            fret: 7
        })); // E
        assert!(coords.contains(&NoteCoordinate {
            string_index: 4,
            fret: 10
        })); // G

        // Verify no false positives (e.g., F note at 1st fret of low E string)
        assert!(!coords.contains(&NoteCoordinate {
            string_index: 5,
            fret: 1
        }));
    }
}
