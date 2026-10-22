mod cli;

use std::io::IsTerminal;
use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Commands, LabelArg};
use core_engine::chord_parser::{parse_chord_parts, parse_chord_symbol};
use core_engine::cli_engine::{
    build_scale_map, create_fretboard_config, decide_use_flats, format_scale_notes,
    generate_ranked_voicings, print_chord_intervals, resolve_fret_count, resolve_instrument_preset,
};
use core_engine::cli_error::CliError;
use core_engine::instrument::Orientation;
use core_engine::output::{format_voicing_w_playability, fret_window};
use core_engine::render::{ColorScheme, LabelMode, RenderOptions, render_fretboard};
use core_engine::theory::chord::Chord;
use core_engine::theory::note::NoteSpelling;
use core_engine::voicing::{VoicingConfig, muted_strings, voicing_to_coords};
use core_engine::voicing_ranker::RankingStrategy::MostPlayable;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Bundles the args of a Chord command request
struct ChordRequest {
    args: Vec<String>,
    intervals: bool,
    instrument: String,
    voicing_count: usize,
    max_stretch: u8,
    capo: u8,
    frets: Option<u8>,
    label: LabelArg,
    no_color: bool,
}

fn label_mode(label: LabelArg) -> LabelMode {
    match label {
        LabelArg::Dot => LabelMode::Dot,
        LabelArg::Note => LabelMode::NoteName,
        LabelArg::Interval => LabelMode::Interval,
    }
}

fn color_enabled(no_color_flag: bool, no_color_env: bool, is_tty: bool) -> bool {
    !no_color_flag && !no_color_env && is_tty
}

fn use_color(no_color_flag: bool) -> bool {
    // Check if the output is a terminal and if the NO_COLOR environment variable is set
    color_enabled(
        no_color_flag,
        std::env::var_os("NO_COLOR").is_some(),
        std::io::stdout().is_terminal(),
    )
}
fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Chord {
            args,
            intervals,
            instrument,
            voicings,
            label,
            no_color,
        } => run_chord(ChordRequest {
            args,
            intervals,
            instrument,
            voicing_count: voicings,
            max_stretch: cli.max_stretch,
            capo: cli.capo,
            frets: cli.frets,
            label,
            no_color,
        }),

        Commands::Identify { shape, instrument } => {
            // TODO: implement
            eprintln!(
                "Identify command has not been implemented yet: shape={:?}, instrument={}",
                shape, instrument
            );
            Ok(())
        }

        Commands::Scale {
            root,
            scale_type,
            instrument,
        } => run_scale(&root, &scale_type, &instrument),
    }
}

fn run_chord(req: ChordRequest) -> Result<(), CliError> {
    let ChordRequest {
        args,
        intervals,
        instrument,
        voicing_count,
        max_stretch,
        capo,
        frets,
        label,
        no_color,
    } = req;

    let parsed = match args.as_slice() {
        [symbol] => parse_chord_symbol(symbol),
        [root, quality] => parse_chord_parts(root, quality, None),
        [root, quality, extension] => parse_chord_parts(root, quality, Some(extension)),
        _ => unreachable!("clap restricts chord args to 1-3 elements"),
    }
    .map_err(|error| CliError::InvalidChord(error.to_string()))?;

    let (root, chord_type) = parsed;
    let chord = Chord::new(root, chord_type);

    if intervals {
        print_chord_intervals(&chord);
        return Ok(());
    }

    // Decide enharmonic spelling for the chord notes based on the input
    let prefer_flat = decide_use_flats(&args);

    let note_names = chord
        .notes
        .iter()
        .map(|&note| {
            if prefer_flat {
                NoteSpelling::prefer_flat(note).display()
            } else {
                NoteSpelling::prefer_sharp(note).display()
            }
        })
        .collect::<Vec<_>>();

    println!("Chord: {}", note_names.join(", "));

    // Instrument and tuning
    let preset = resolve_instrument_preset(&instrument)?;
    let fret_count = resolve_fret_count(&preset, frets, capo)?;
    let fretboard_config =
        create_fretboard_config(preset.tuning, fret_count, Orientation::RightHanded);
    let voicing_config = VoicingConfig {
        max_stretch,
        capo_fret: capo,
        require_all_target_notes: true,
        allow_open_strings: true,
    };

    let ranked_voicings =
        generate_ranked_voicings(&chord, &fretboard_config, &voicing_config, MostPlayable);

    if ranked_voicings.is_empty() {
        return Err(CliError::NoVoicings);
    }

    let min_span = max_stretch.saturating_add(1);
    let scheme = use_color(no_color).then(ColorScheme::default);
    let label_mode = label_mode(label);

    println!("Top {} Voicings:", voicing_count);
    for (index, scored) in ranked_voicings.iter().take(voicing_count).enumerate() {
        println!(
            "{}. Score {} | {}",
            index + 1,
            scored.score,
            format_voicing_w_playability(&scored.voicing, &fretboard_config.tuning, prefer_flat)
        );

        let options = RenderOptions {
            fret_range: fret_window(&scored.voicing, capo, fret_count, min_span),
            root: Some(root),
            use_flats: prefer_flat,
            capo,
            label_mode,
            show_inlays: true,
            scheme,
            muted_strings: muted_strings(&scored.voicing),
        };
        println!(
            "{}",
            render_fretboard(
                &fretboard_config,
                &voicing_to_coords(&scored.voicing),
                &options
            )
        );
    }

    Ok(())
}

fn run_scale(root: &str, scale_name: &str, instrument: &str) -> Result<(), CliError> {
    let scale_map = build_scale_map(root, scale_name, instrument, 24)?;
    println!("Scale: {}", format_scale_notes(&scale_map));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_requires_tty_and_no_opt_out() {
        assert!(color_enabled(false, false, true));
        assert!(!color_enabled(true, false, true));
        assert!(!color_enabled(false, true, true));
        assert!(!color_enabled(false, false, false));
    }

    #[test]
    fn label_arg_maps_to_label_mode() {
        assert_eq!(label_mode(LabelArg::Note), LabelMode::NoteName);
        assert_eq!(label_mode(LabelArg::Dot), LabelMode::Dot);
        assert_eq!(label_mode(LabelArg::Interval), LabelMode::Interval);
    }
}
