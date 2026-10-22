use crate::cli_error::CliError;
use crate::instrument::registry::{get_instrument_preset, get_tuning};
use crate::instrument::{FretBoardConfig, InstrumentPreset, Orientation, Tuning};
use crate::mapping::{NoteCoordinate, find_notes_on_fretboard};
use crate::render::MAX_FRET;
use crate::theory::Note;
use crate::theory::chord::Chord;
use crate::theory::note::NoteSpelling;
use crate::theory::scale::{ScaleType, generate_scale};
use crate::voicing::{VoicingConfig, generate_voicing};
use crate::voicing_ranker::{RankingStrategy, VoicingScore, rank_voicings};
use std::str::FromStr;

pub fn resolve_instrument_preset(key: &str) -> Result<InstrumentPreset, CliError> {
    get_instrument_preset(key).ok_or_else(|| CliError::UnknownTuning(key.to_string()))
}
/// Resolve a tuning registry key such as "guitar/standard" into a tuning.
pub fn resolve_tuning(key: &str) -> Result<Tuning<'static>, CliError> {
    get_tuning(key).ok_or_else(|| CliError::UnknownTuning(key.to_string()))
}

/// Fret count to use. If provided use the user's override, else use the instrument's own fret count.
/// Rejects counts above the renderer limit and a capo beyond the last fret.
pub fn resolve_fret_count(
    preset: &InstrumentPreset,
    requested: Option<u8>,
    capo: u8,
) -> Result<u8, CliError> {
    let fret_count = requested.unwrap_or(preset.instrument.fret_count);

    if fret_count > MAX_FRET {
        return Err(CliError::InvalidFret {
            fret: fret_count,
            max: MAX_FRET,
        });
    }
    if capo > fret_count {
        return Err(CliError::InvalidFret {
            fret: capo,
            max: fret_count,
        });
    }

    Ok(fret_count)
}

/// Flats are preferred when an argument spells one (root flat, "b5", "b9", ...) or the
/// quality is the word "minor", unless the root itself is sharp: an explicit sharp root wins.
pub fn decide_use_flats(args: &[String]) -> bool {
    let Some(root) = args.first() else {
        return false;
    };

    if root.as_bytes().get(1) == Some(&b'#') {
        return false;
    }

    // A closure used to skip the root letter so a root of "B"/"b" isn't read as a flat.
    let spells_flat = |s: &str| s.contains('b') || s.to_ascii_lowercase().contains("minor");

    spells_flat(root.get(1..).unwrap_or("")) || args[1..].iter().any(|arg| spells_flat(arg))
}

/// Create a FretBoardConfig which will be used for mapping notes to frets, and for generating voicings.
pub fn create_fretboard_config(
    tuning: Tuning<'static>,
    fret_count: u8,
    orientation: Orientation,
) -> FretBoardConfig<'static> {
    FretBoardConfig::new(tuning, fret_count, orientation)
}

/// Get the Pitch of each open string in the given tuning.
pub fn tuning_open_string_pitches(tuning: &Tuning<'_>) -> Vec<u8> {
    tuning.strings.iter().map(|midi| midi.0 % 12).collect()
}

/// Parse a scale name such as "major" or "minor_pentatonic" into a ScaleType.
pub fn parse_scale(name: &str) -> Result<ScaleType, CliError> {
    match name.to_ascii_lowercase().as_str() {
        "major" | "ionian" => Ok(ScaleType::Major),
        "minor" | "natural_minor" | "aeolian" => Ok(ScaleType::NaturalMinor),
        "major_pentatonic" => Ok(ScaleType::MajorPentatonic),
        "minor_pentatonic" => Ok(ScaleType::MinorPentatonic),
        _ => Err(CliError::UnknownScale(name.to_string())),
    }
}

/// The result of the scale pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleMap {
    pub notes: Vec<Note>,
    pub coords: Vec<NoteCoordinate>,
    pub tuning: Tuning<'static>,
    pub prefer_flat: bool,
}

/// Scale pipeline:
/// root (str) -> Note -> ScaleType -> generate_scale -> tuning -> find_notes_on_fretboard
pub fn build_scale_map(
    root: &str,
    scale_name: &str,
    tuning_key: &str,
    fret_count: u8,
) -> Result<ScaleMap, CliError> {
    let root_note = Note::from_str(root).map_err(|_| CliError::InvalidNote(root.to_string()))?;
    let scale_type = parse_scale(scale_name)?;
    let tuning = resolve_tuning(tuning_key)?;

    let scale_notes = generate_scale(root_note, scale_type);
    let config = create_fretboard_config(tuning, fret_count, Orientation::RightHanded);
    let coords = find_notes_on_fretboard(&scale_notes, &config);

    Ok(ScaleMap {
        notes: scale_notes,
        coords,
        tuning,
        prefer_flat: root.as_bytes().get(1) == Some(&b'b'),
    })
}

fn spell_note(note: Note, prefer_flat: bool) -> String {
    if prefer_flat {
        NoteSpelling::prefer_flat(note).display()
    } else {
        NoteSpelling::prefer_sharp(note).display()
    }
}

/// Format the notes of the scale separated by commas, using the preferred spelling (flat or sharp) based on the root note.
pub fn format_scale_notes(scale_map: &ScaleMap) -> String {
    scale_map
        .notes
        .iter()
        .map(|&note| spell_note(note, scale_map.prefer_flat))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Chord pipeline:
/// Chord -> find_notes_on_fretboard -> generate_voicings -> rank_voicings
///
/// An empty result is valid, and indicates that no voicings were found for the given chord on the given fretboard configuration.
/// The CLI will handle this case and display a message to the user.
pub fn generate_ranked_voicings(
    chord: &Chord,
    fret_board_config: &FretBoardConfig<'_>,
    voicing_config: &VoicingConfig,
    ranking_strategy: RankingStrategy,
) -> Vec<VoicingScore> {
    let coords = find_notes_on_fretboard(&chord.notes, fret_board_config);
    let open_strings = tuning_open_string_pitches(&fret_board_config.tuning);
    let string_count = fret_board_config.tuning.strings.len();

    let voicings = generate_voicing(
        &chord.notes,
        &coords,
        string_count,
        &open_strings,
        voicing_config,
    );

    rank_voicings(voicings, ranking_strategy)
}

pub fn print_chord_intervals(chord: &Chord) {
    let mut interval_names = vec!["Root".to_string()];

    // Add the intervals from the chord quality to the list of interval names
    interval_names.extend(
        chord
            .chord_type
            .quality
            .intervals()
            .iter()
            .map(|interval| format!("{interval:?}")),
    );
    // Add the intervals from the chord extension to the list of interval names
    interval_names.extend(
        chord
            .chord_type
            .extension
            .intervals()
            .iter()
            .map(|interval| format!("{interval:?}")),
    );
    println!("{}", interval_names.join(", "));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    fn guitar() -> InstrumentPreset {
        get_instrument_preset("guitar/standard").unwrap()
    }

    #[test]
    fn flat_root_prefers_flats() {
        assert!(decide_use_flats(&s(&["Ebmaj7"])));
    }

    #[test]
    fn plain_root_prefers_sharps() {
        assert!(!decide_use_flats(&s(&["A"])));
    }

    #[test]
    fn minor_word_prefers_flats() {
        assert!(decide_use_flats(&s(&["G", "minor"])));
    }

    #[test]
    fn minor_word_is_case_insensitive() {
        assert!(decide_use_flats(&s(&["b", "Minor"])));
    }

    #[test]
    fn sharp_root_wins_over_flat_extension_and_minor_word() {
        assert!(!decide_use_flats(&s(&["C#m7b5"])));
        assert!(!decide_use_flats(&s(&["C#", "minor"])));
    }

    #[test]
    fn fret_count_defaults_to_instrument() {
        assert_eq!(resolve_fret_count(&guitar(), None, 0), Ok(24));
    }

    #[test]
    fn fret_count_override_wins() {
        assert_eq!(resolve_fret_count(&guitar(), Some(12), 0), Ok(12));
    }

    #[test]
    fn fret_count_above_renderer_limit_rejected() {
        assert!(resolve_fret_count(&guitar(), Some(99), 0).is_err());
    }

    #[test]
    fn capo_beyond_last_fret_rejected_with_real_max() {
        assert_eq!(
            resolve_fret_count(&guitar(), None, 30),
            Err(CliError::InvalidFret { fret: 30, max: 24 })
        );
        assert_eq!(
            resolve_fret_count(&guitar(), Some(12), 13),
            Err(CliError::InvalidFret { fret: 13, max: 12 })
        );
        assert_eq!(resolve_fret_count(&guitar(), Some(12), 12), Ok(12));
    }

    #[test]
    fn b_root_is_not_flat() {
        assert!(!decide_use_flats(&s(&["B"])));
        assert!(!decide_use_flats(&s(&["b"])));
        assert!(!decide_use_flats(&s(&["Bmaj7"])));
    }

    #[test]
    fn bb_root_is_flat() {
        assert!(decide_use_flats(&s(&["Bb"])));
        assert!(decide_use_flats(&s(&["bb"])));
    }
}
