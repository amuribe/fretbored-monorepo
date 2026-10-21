use crate::instrument::{
    BassTunings, GuitarTunings, Instrument, InstrumentPreset, StandardTunings, Tuning,
};
pub const TUNING_KEYS: &[&str] = &[
    "guitar/standard",
    "guitar/drop_d",
    "guitar/eb_standard",
    "guitar/full_step_down",
    "bass/standard",
    "bass/drop_d",
    "banjo/standard",
    "mandolin/standard",
    "ukulele/standard",
];

pub fn get_instrument_preset(key: &str) -> Option<InstrumentPreset> {
    let (instrument, tuning) = match key {
        "guitar/standard" => (Instrument::GUITAR, StandardTunings::guitar_standard()),
        "guitar/drop_d" => (Instrument::GUITAR, GuitarTunings::guitar_drop_d()),
        "guitar/eb_standard" => (Instrument::GUITAR, GuitarTunings::guitar_eb_standard()),
        "guitar/full_step_down" => (Instrument::GUITAR, GuitarTunings::guitar_full_step_down()),
        "bass/standard" => (Instrument::BASS, StandardTunings::bass_standard()),
        "bass/drop_d" => (Instrument::BASS, BassTunings::bass_drop_d()),
        "banjo/standard" => (
            Instrument::BANJO_5_STRING,
            StandardTunings::banjo_standard(),
        ),
        "mandolin/standard" => (Instrument::MANDOLIN, StandardTunings::mandolin_standard()),
        "ukulele/standard" => (Instrument::UKULELE, StandardTunings::ukulele_standard()),
        _ => return None,
    };
    Some(InstrumentPreset { instrument, tuning })
}
pub fn get_tuning(key: &str) -> Option<Tuning<'static>> {
    get_instrument_preset(key).map(|p| p.tuning)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_internally_consistent() {
        use crate::render::MAX_FRET;
        for key in TUNING_KEYS {
            let p = get_instrument_preset(key).unwrap();
            let i = p.instrument;
            assert_eq!(
                i.string_count(),
                p.tuning.strings.len(),
                "{key}: string count"
            );
            assert!((1..=MAX_FRET).contains(&i.fret_count), "{key}: fret_count");
            assert!(
                i.first_fret.iter().all(|&f| f <= i.fret_count),
                "{key}: first_fret"
            );
        }
    }
}
