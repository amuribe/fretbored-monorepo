use crate::instrument::tuning::{BassTunings, GuitarTunings, StandardTunings, Tuning};

pub fn get_tuning(key: &str) -> Option<Tuning<'static>> {
    match key {
        // GUITAR TUNINGS
        "guitar/standard" => Some(StandardTunings::guitar_standard()),
        "guitar/drop_d" => Some(GuitarTunings::guitar_drop_d()),
        "guitar/eb_standard" => Some(GuitarTunings::guitar_eb_standard()),
        "guitar/full_step_down" => Some(GuitarTunings::guitar_full_step_down()),

        // BASS TUNINGS
        "bass/standard" => Some(StandardTunings::bass_standard()),
        "bass/drop_d" => Some(BassTunings::bass_drop_d()),

        // BANJO TUNINGS
        "banjo/standard" => Some(StandardTunings::banjo_standard()),

        // MANDOLIN TUNINGS
        "mandolin/standard" => Some(StandardTunings::mandolin_standard()),

        // UKULELE TUNINGS
        "ukulele/standard" => Some(StandardTunings::ukulele_standard()),

        _ => None,
    }
}
