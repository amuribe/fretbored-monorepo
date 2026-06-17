mod cli;
use clap::Parser;
use cli::{Cli, Commands};

use core_engine::chord_parser::{parse_chord_parts, parse_chord_symbol};
use core_engine::theory::chord::Chord;
use core_engine::theory::note::NoteSpelling;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Chord { args, intervals } => {
            // Route the input based on the number of arguments provided
            let parse_result = match args.len() {
                1 => parse_chord_symbol(&args[0]),
                2 => parse_chord_parts(&args[0], &args[1], None),
                3 => parse_chord_parts(&args[0], &args[1], Some(&args[2])),
                _ => unreachable!("Clap restricts args to 1..=3"),
            };

            let (root, chord_type) = match parse_result {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            // Generate the chord
            let chord = Chord::new(root, chord_type);

            // Output logic
            if intervals {
                let mut interval_names = vec!["Root".to_string()];

                for interval in chord.chord_type.quality.intervals() {
                    interval_names.push(format!("{:?}", interval));
                }
                for interval in chord.chord_type.extension.intervals() {
                    interval_names.push(format!("{:?}", interval));
                }

                println!("{}", interval_names.join(", "));
            } else {
                // Use flats if 'm' or 'b' exists in the input strings
                let raw_input = args.join(" ");
                let prefer_flat = raw_input.contains('m') || raw_input.contains('b');

                let note_names: Vec<String> = chord
                    .notes
                    .iter()
                    .map(|&n| {
                        if prefer_flat {
                            NoteSpelling::prefer_flat(n).display()
                        } else {
                            NoteSpelling::prefer_sharp(n).display()
                        }
                    })
                    .collect();

                println!("{}", note_names.join(", "));
            }
        }
    }
}
