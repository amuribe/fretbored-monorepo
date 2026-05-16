use crate::theory::{MidiNote, Note};
use serde::Deserialize;

// The notes of each string on the instrument and the corresponding midi numbers
#[derive(Debug, Deserialize, PartialEq)]
pub struct Tuning {
    pub name: String,
    pub strings: Vec<MidiNote>,
}

impl Tuning {
    // Get the note produce by pressing a specified fret on a specific string
    pub fn note_at_fret(&self, string_index: usize, fret: u8) -> Option<Note> {
        // '?' returns null if out of bounds
        let open_midi = self.strings.get(string_index)?;
        let fretted_midi = open_midi.0 + fret;
        // Get note from midi value of fretted note
        Some(Note::from_midi(fretted_midi))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Instrument {
    pub name: String,
    pub tunings: Vec<Tuning>,
}

// Holds all locally defined instruments and tunings

pub struct InstrumentDatabase {
    pub instruments: Vec<Instrument>,
}

impl InstrumentDatabase {
    // Parse the JSON file instruments.json at compile-time
    pub fn load() -> Self {
        let json_data = include_str!("data/instruments.json");

        // Convert string to ChordTemplate
        let instruments: Vec<Instrument> =
            serde_json::from_str(json_data).expect("Failed to parse instruments.json");

        Self { instruments }
    }

    // Lookup instrument(optional) and tuning
    pub fn get_tuning(&self, instrument_name: &str, tuning_name: &str) -> Option<&Tuning> {
        self.instruments
            .iter()
            .find(|inst| inst.name == instrument_name)?
            .tunings
            .iter()
            .find(|tun| tun.name == tuning_name)
    }
}

// Test Tuning, Instrument, and InstrumentDatabase
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_load_and_query() -> Result<(), String> {
        let db = InstrumentDatabase::load();

        // Retrieve the standard guitar tuning from JSON
        let guitar_standard = db
            .get_tuning("Guitar", "Standard")
            .ok_or_else(|| "Failed to find Guitar Standard tuning from database".to_string())?;

        // Test JSON values are valid
        assert_eq!(guitar_standard.name, "Standard");
        assert_eq!(guitar_standard.strings.len(), 6);

        // Test note at fret
        // String 0 (low E(E2)) 3rd fret = Note::G = G2 = MIDI 43
        assert_eq!(guitar_standard.note_at_fret(0, 3), Some(Note::G));
        // String 5 (high E(E4)) open string(fret 0) = Note::E = E4 = MIDI 64
        assert_eq!(guitar_standard.note_at_fret(5, 0), Some(Note::E));

        // Out of bounds fret
        assert_eq!(guitar_standard.note_at_fret(99, 0), None);

        // On success
        Ok(())
    }
}
