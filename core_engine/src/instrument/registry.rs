use crate::instrument::tuning::{GuitarTunings, StandardTunings, Tuning};
use std::collections::HashMap;

// Index instruments and tuning

// Tunings don't change so marked as static
pub struct TuningRegistry {
    tunings: HashMap<&'static str, Tuning<'static>>,
}

impl TuningRegistry {
    pub fn new() -> Self {
        let mut tunings = HashMap::new();

        // String key consists of <instrument-name>/<tuning-name>

        // Insert tunings

        // GUITAR TUNINGS
        tunings.insert("guitar/standard", StandardTunings::guitar_standard());
        tunings.insert("guitar/drop_d", GuitarTunings::guitar_drop_d());
        // BASS TUNINGS
        tunings.insert("bass/standard", StandardTunings::bass_standard());
        // UKULELE TUNINGS
        tunings.insert("ukulele/standard", StandardTunings::ukulele_standard());

        Self { tunings }
    }

    pub fn get(&self, key: &str) -> Option<&Tuning<'static>> {
        self.tunings.get(key)
    }

    // Return the full list of keys in the registry
    pub fn keys(&self) -> Vec<&'static str> {
        self.tunings.keys().copied().collect()
    }
}
