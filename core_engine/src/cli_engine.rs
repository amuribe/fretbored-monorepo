use crate::cli_error::CliError;
use crate::instrument::registry::get_tuning;
use crate::instrument::{FretBoardConfig, Orientation, Tuning};
use crate::mapping::find_notes_on_fretboard;
use crate::theory::chord::Chord;
use crate::theory::scale::ScaleType;
use crate::voicing::{VoicingConfig, generate_voicing};
use crate::voicing_ranker::{RankingStrategy, VoicingScore, rank_voicings};

/// Resolve a tuning registry key such as "guitar/standard" into a tuning.
pub fn resolve_tuning(key: &str) -> Result<Tuning<'static>, CliError> {
    get_tuning(key).ok_or_else(|| CliError::UnknownTuning(key.to_string()))
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
        "major | ionian" => Ok(ScaleType::Major),
        "minor | natural_minor | aeolian" => Ok(ScaleType::NaturalMinor),
        "major_pentatonic" => Ok(ScaleType::MajorPentatonic),
        "minor_pentatonic" => Ok(ScaleType::MinorPentatonic),
        _ => Err(CliError::UnknownScale(name.to_string())),
    }
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
