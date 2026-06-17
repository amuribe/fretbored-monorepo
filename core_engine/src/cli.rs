use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fretbored")]
#[command(about = "High-performance music theory engine for string instruments")]
pub struct Cli {
    #[arg(short = 's', long, default_value_t = 4, global = true)]
    pub max_stretch: u8,

    #[arg(short = 'c', long, default_value_t = 0, global = true)]
    pub capo: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a chord from a single symbol ("Cm7") or parts ("C" "minor" "7")
    Chord {
        /// The chord symbol or separated root, quality, and extension
        #[arg(required = true, num_args = 1..=3)]
        args: Vec<String>,

        /// Output the intervals whihc build the chord instead of pitches
        #[arg(short, long)]
        intervals: bool,
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
        let args = vec!["fretbored", "chord", "Cm7", "-s", "5"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.max_stretch, 5);

        let Commands::Chord {
            args: parsed_args,
            intervals,
        } = cli.command;
        assert_eq!(parsed_args, vec!["Cm7"]);
        assert!(!intervals);
    }

    #[test]
    fn parse_separated_args_command() {
        let args = vec!["fretbored", "chord", "G", "minor", "9", "-c", "2"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.capo, 2);

        let Commands::Chord {
            args: parsed_args,
            intervals,
        } = cli.command;
        assert_eq!(parsed_args, vec!["G", "minor", "9"]);
        assert!(!intervals);
    }

    #[test]
    fn parse_missing_required_args_fails() {
        let args = vec!["fretbored", "chord"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }
}
