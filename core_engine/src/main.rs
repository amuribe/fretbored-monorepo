mod cli;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Commands};
use core_engine::chord_parser::{parse_chord_parts, parse_chord_symbol};
use core_engine::cli_engine::{
    build_scale_map, create_fretboard_config, format_scale_notes, generate_ranked_voicings,
    print_chord_intervals, resolve_tuning,
};
use core_engine::cli_error::CliError;
use core_engine::instrument::Orientation;
use core_engine::output::format_voicing_w_playability;
use core_engine::theory::chord::Chord;
use core_engine::theory::note::NoteSpelling;
use core_engine::voicing::VoicingConfig;
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

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Chord {
            args,
            intervals,
            instrument,
            voicings,
        } => run_chord(
            args,
            intervals,
            instrument,
            voicings,
            cli.max_stretch,
            cli.capo,
        ),

        Commands::Identify { frets, instrument } => {
            // TODO: implement
            eprintln!(
                "Identify command has not been implemented yet: frets={:?}, instrument={}",
                frets, instrument
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

fn run_chord(
    args: Vec<String>,
    intervals: bool,
    instrument: String,
    voicing_count: usize,
    max_stretch: u8,
    capo: u8,
) -> Result<(), CliError> {
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
    let prefer_flat = args.iter().any(|arg| arg.contains('b') || arg == "minor");

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

    let tuning = resolve_tuning(&instrument)?;
    let fretboard_config = create_fretboard_config(tuning, 24, Orientation::RightHanded);
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

    println!("Top {} Voicings:", voicing_count);
    for (index, scored) in ranked_voicings.iter().take(voicing_count).enumerate() {
        println!(
            "{}. Score {} | {}",
            index + 1,
            scored.score,
            format_voicing_w_playability(&scored.voicing, &fretboard_config.tuning,)
        );
    }

    Ok(())
}

fn run_scale(root: &str, scale_name: &str, instrument: &str) -> Result<(), CliError> {
    let scale_map = build_scale_map(root, scale_name, instrument, 24)?;
    println!("Scale: {}", format_scale_notes(&scale_map));

    Ok(())
}
