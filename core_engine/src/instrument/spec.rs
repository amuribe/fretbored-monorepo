use crate::instrument::Tuning;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instrument {
    pub name: &'static str,
    pub fret_count: u8,
    // Fret position where each string begins, in tuning order (low to high index).
    // 0 for normal open strings, 5 for 5th string banjo, etc.
    pub first_fret: &'static [u8],
}

impl Instrument {
    pub const GUITAR: Self = Self {
        name: "Guitar",
        fret_count: 24,
        first_fret: &[0; 6],
    };
    pub const BASS: Self = Self {
        name: "Bass",
        fret_count: 24,
        first_fret: &[0; 4],
    };
    // Typical soprano ukulele fret count
    pub const UKULELE: Self = Self {
        name: "Ukulele",
        fret_count: 12,
        first_fret: &[0; 4],
    };
    pub const BANJO_5_STRING: Self = Self {
        name: "5-String Banjo",
        fret_count: 22,
        first_fret: &[5, 0, 0, 0, 0],
    };
    pub const BANJO_4_STRING: Self = Self {
        name: "4-String Banjo",
        fret_count: 22,
        first_fret: &[0; 4],
    };
    pub const MANDOLIN: Self = Self {
        name: "Mandolin",
        fret_count: 24,
        first_fret: &[0; 4],
    };

    pub const fn string_count(&self) -> usize {
        self.first_fret.len()
    }
}

// An instrument paired with one of its tunings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstrumentPreset {
    pub instrument: Instrument,
    pub tuning: Tuning<'static>,
}
