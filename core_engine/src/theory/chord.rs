use crate::theory::Note;
use serde::Deserialize;
use std::collections::HashSet;

// A chord template defined in JSON file chords.json
#[derive(Debug, Deserialize, PartialEq)]
pub struct ChordTemplate {
    pub name: String,
    pub intervals: Vec<u8>,
}

// Holds all locally defined chords

pub struct ChordDatabase {
    pub templates: Vec<ChordTemplate>,
}

impl ChordDatabase {
    // Load JSON at compile time
    pub fn load() -> Self {
        let json_data = include_str!("data/chords.json");

        // Convert string to ChordTemplate
        let templates: Vec<ChordTemplate> =
            serde_json::from_str(json_data).expect("Failed to parse chords.json");

        Self { templates }
    }
}

// Given the absolute MIDI values of the notes in a chord, convert them into unique integers 0-11
fn get_unique_pitch_classes(midi_notes: &[u8]) -> HashSet<u8> {
    // Iterate over notes and map them to a valid chromatic range using modulo 12
    // |&note| dereferences note to get value
    midi_notes.iter().map(|&note| note % 12).collect()
}

// Calculate distance from root to all other notes in the set(chord) and return the sorted intervals in the chord
fn calculate_intervals(root: u8, pitches: &HashSet<u8>) -> Vec<u8> {
    // Mut because we must sort in place
    // Move note above root then sub root to get distance (modulo 12 to wrap)
    let mut intervals: Vec<u8> = pitches
        .iter()
        .map(|&note| (note + 12 - root) % 12)
        .collect();

    intervals.sort_unstable();
    intervals
}

// Identify chord by trying every pitch as a root and looking for a matching pattern

pub fn identify_chord(midi_notes: &[u8], db: &ChordDatabase) -> Option<String> {
    let pitches = get_unique_pitch_classes(midi_notes);

    // Loop through every pitch treating it as root
    for &root in &pitches {
        // Get intervals relative to root
        let intervals = calculate_intervals(root, &pitches);

        // Loop through chord templates to find a match
        for template in &db.templates {
            if intervals == template.intervals {
                // If template match is found return formatted chord string
                return Some(format!("{:?} {}", Note::from_midi(root), template.name));
            }
        }
    }

    // Return None if no matching template is found
    None
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn test_load_database() {
        let db = ChordDatabase::load();
        // Verify it loaded the correct amount of chords
        assert_eq!(db.templates.len(), 7);

        // Verify the first chord is a fifth chord
        assert_eq!(db.templates[0].name, "5");
        assert_eq!(db.templates[0].intervals, vec![0, 7]);

        // Verify the second chord is Major
        assert_eq!(db.templates[1].name, "Major");
        assert_eq!(db.templates[1].intervals, vec![0, 4, 7]);
    }

    #[test]
    fn test_chord_identification() {
        let db = ChordDatabase::load();

        // Test C Major triad: C (60), E (64), G (67)
        let c_major = vec![60, 64, 67];
        assert_eq!(identify_chord(&c_major, &db), Some("C Major".to_string()));

        // Test C Major inversion: E (64), G (67), C (72)
        let c_major_inversion = vec![64, 67, 72];
        assert_eq!(
            identify_chord(&c_major_inversion, &db),
            Some("C Major".to_string())
        );
    }
}
