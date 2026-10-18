use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "fretbored",
    about = "High-performance music theory engine for string instruments",
    after_help = "EXAMPLES:\n  \
    Generate C major chord on standard guitar:\n    \
    fretbored chord C major\n  \
    Generate Gm7b5 with tight hand stretch:\n    \
    fretbored chord Gm7b5 --max-stretch 3\n  \
    Show 10 best voicings for Cmaj7:\n    \
    fretbored chord Cmaj7 --voicings 10\n  \
    Use a capo on fret 2:\n    \
    fretbored chord Am --capo 2\n  \
    Display intervals instead of notes:\n    \
    fretbored chord Dm7 --intervals"
)]
pub struct Cli {
    /// Maximum distance between the lowest and highest fretted notes in a voicing
    #[arg(short = 's', long, default_value_t = 4, global = true)]
    pub max_stretch: u8,

    /// Capo position. Zero means no capo.
    #[arg(short = 'c', long, default_value_t = 0, global = true)]
    pub capo: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate chord notes and playable voicings.
    Chord {
        /// A chord symbol, or root, quality, and extension as separate arguments.
        ///
        /// Examples: Cmaj7, C major 7
        #[arg(required = true, num_args = 1..=3)]
        args: Vec<String>,

        /// Print the intervals of the chord instead of the notes.
        #[arg(short = 'I', long)]
        intervals: bool,

        /// Instrument tuning registry key, e.g. "guitar/standard" or "banjo/standard".
        #[arg(short = 'i', long, default_value = "guitar/standard")]
        instrument: String,

        /// Number of ranked voicings to display
        #[arg(short = 'n', long, default_value_t = 5)]
        voicings: usize,
    },

    /// Identify a chord from a comma separated fret shape.
    Identify {
        /// Frets as CSV, Example(Open G on guitar e standard): "3,2,0,0,3,3".
        #[arg(required = true)]
        frets: String,

        /// Intrument tuning registry key, e.g. "guitar/standard" or "banjo/standard".
        #[arg(short = 'i', long, default_value = "guitar/standard")]
        instrument: String,
    },

    /// Display a scale on the fretboard
    Scale {
        /// Root note, Example: "C", "A", "Bb", "F#"
        #[arg(required = true)]
        root: String,

        /// Scale type: major, minor, pentatonic
        #[arg(required = true)]
        scale_type: String,

        /// Instrument tuning registry key, e.g. "guitar/standard" or "banjo/standard".
        #[arg(short = 'i', long, default_value = "guitar/standard")]
        instrument: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_app() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_single_symbol_command() {
        let cli = Cli::try_parse_from([
            "fretbored",
            "chord",
            "Cm7",
            "--max-stretch",
            "5",
            "--instrument",
            "guitar/standard",
        ])
        .unwrap();

        assert_eq!(cli.max_stretch, 5);

        match cli.command {
            Commands::Chord {
                args,
                intervals,
                instrument,
                voicings,
            } => {
                assert_eq!(args, vec!["Cm7"]);
                assert!(!intervals);
                assert_eq!(instrument, "guitar/standard");
                assert_eq!(voicings, 5);
            }
            _ => panic!("expected chord command"),
        }
    }

    #[test]
    fn parse_separated_args_command() {
        let cli =
            Cli::try_parse_from(["fretbored", "chord", "G", "minor", "9", "--capo", "2"]).unwrap();

        assert_eq!(cli.capo, 2);

        match cli.command {
            Commands::Chord { args, .. } => {
                assert_eq!(args, vec!["G", "minor", "9"]);
            }
            _ => panic!("expected chord command"),
        }
    }

    #[test]
    fn parse_missing_required_args_fails() {
        let result = Cli::try_parse_from(["fretbored", "chord"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_intervals_and_instrument_short_flags() {
        let cli = Cli::try_parse_from(["fretbored", "chord", "Cmaj7", "-I", "-i", "guitar/drop_d"])
            .unwrap();

        match cli.command {
            Commands::Chord {
                intervals,
                instrument,
                ..
            } => {
                assert!(intervals);
                assert_eq!(instrument, "guitar/drop_d");
            }
            _ => panic!("expected chord command"),
        }
    }
}
